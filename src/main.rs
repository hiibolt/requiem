mod background;
mod chat;
mod ettethread_ai;

use crate::background::*;
use crate::chat::*;
use crate::ettethread_ai::*;

use bevy::{
    prelude::*,
    window::*,
    asset::Handle,
};
use std::fs;
use std::vec::IntoIter;
use std::collections::HashMap;
use std::env;

use regex::Regex;

use json::parse;
use serde::{ Serialize, Deserialize };

#[derive(Component)]
struct Object {
    r#type: String,
    id: String
}

#[derive(Resource, Default)]
pub struct VisualNovelState {
    // Player-designated constants
    playername: String,
    api_key: String,

    gui_sprites: HashMap<String, Handle<Image>>,

    all_script_transitions: HashMap<String, Vec<Transition>>,
    transitions_iter: IntoIter<Transition>,
    current_scene_id: String,
    
    extra_transitions: Vec<Transition>,

    past_messages: Vec<Message>,

    blocking: bool,
}






/*
      _                          _
  ___| |__   __ _ _ __ __ _  ___| |_ ___ _ __
 / __| '_ \ / _` | '__/ _` |/ __| __/ _ | '__|
| (__| | | | (_| | | | (_| | (__| ||  __| |
 \___|_| |_|\__,_|_|  \__,_|\___|\__\___|_|
*/
/* Components */
#[derive(Component)]
pub struct Character {
    name: String,
    outfit: String,
    emotion: String,
    description: String,
    emotions: Vec<String>
}
#[derive(Component)]
struct CharacterSprites {
    outfits: HashMap::<String, HashMap<String, Handle<Image>>>,
}

/* Resources */
#[derive(Resource)]
struct OpacityFadeTimer(Timer);

/* Events */
struct EmotionChangeEvent {
    name: String,
    emotion: String
}

pub struct CharacterController;
impl Plugin for CharacterController {
    fn build(&self, app: &mut App){
        app.insert_resource(OpacityFadeTimer(Timer::from_seconds(0.005, TimerMode::Repeating)))
            .add_startup_system(import_characters)
            .add_event::<EmotionChangeEvent>()
            .add_system(update_characters);
    }
}
fn import_characters(mut commands: Commands, asset_server: Res<AssetServer>){
    /* Character Setup */
    // Asset Gathering
    let mut outfits = HashMap::<String, HashMap<String, Handle<Image>>>::new();
    let master_character_dir = std::env::current_dir()
        .expect("Failed to get current directory!")
        .join("assets")
        .join("characters")
        .join("Nayu");
    let outfit_dirs = fs::read_dir(master_character_dir)
        .expect("Unable to read outfit folders!")
        .filter_map(|entry_result| {
            let entry = entry_result.ok()?;
            if let Ok(file_type) = entry.file_type() {
                match file_type.is_dir() {
                    true => return Some(entry.path()),
                    false => return None
                }
            } else {
                None
            }
        });
    for outfit_dir in outfit_dirs {
        let mut emotion_sprites = HashMap::<String, Handle<Image>>::new();
        let outfit_name = outfit_dir
            .file_name().expect("No directory name!")
            .to_str().expect("Malformed UTF-8 in directory name, please verify it meets UTF-8 validity!")
            .to_owned();
        
        let sprite_paths = fs::read_dir(outfit_dir)
            .expect("No character data!")
            .filter_map(|entry| {
                if let Ok(entry) = entry {
                    Some(entry.path())
                }else{
                    info!("Failed to read file data of `{:?}`!", entry);
                    None
                }
            });
        for sprite_path in sprite_paths {
            let sprite_name = sprite_path
                .file_stem().expect("No file name! Your emotion sprites MUST have a label to be able to be referred to!")
                .to_str().expect("Malformed UTF-8 in file name, please verify it meets UTF-8 validity!")
                .to_string();
            let file_texture = asset_server.load(sprite_path);

            println!("Imported sprite '{}' for outfit '{}'", sprite_name, outfit_name);
            emotion_sprites.insert(sprite_name, file_texture);
        }
        outfits.insert(outfit_name, emotion_sprites);
    }
    // Character Info Gathering
    let character_string: String = fs::read_to_string(std::env::current_dir()
            .expect("Failed to get current directory!")
            .join("assets")
            .join("characters")
            .join("Nayu")
            .join("character.json"))
        .expect("Issue reading file!");
    let parsed_character = parse(&character_string).expect("Malformed JSON!");

    let name = parsed_character["name"].as_str().expect("Missing 'name' attribute").to_owned();
    let outfit = parsed_character["default_outfit"].as_str().expect("Missing 'name' attribute").to_owned();
    let emotion = parsed_character["default_emotion"].as_str().expect("Missing 'name' attribute").to_owned();

    commands.spawn((
        Object {
            r#type: String::from("character"),
            id: format!("_character_{name}")
        },
        Character {
            name,
            outfit: outfit.clone(),
            emotion: outfit.clone(),
            description: parsed_character["description"].as_str().expect("Missing 'name' attribute").to_owned(),
            emotions: parsed_character["emotions"]
                .members()
                .map(|entry| entry.as_str()
                    .expect("Missing 'name' attribute")
                    .to_owned()
                ).collect::<Vec<String>>(),
        },
        SpriteBundle {
            texture: outfits.get(&outfit)
                .expect("'{character.outfit}' attribute does not exist!")
                .get(&emotion)
                .expect("'default_emotion' atttribute does not exist!")
                .clone(),
            transform: Transform::IDENTITY
                .with_translation(Vec3 { x:0., y:-40., z:1. } )
                .with_scale(Vec3 { x:0.75, y:0.75, z:1. } ),
            ..default()
        },
        CharacterSprites { outfits }
    ));
}
fn update_characters(
    mut character_query: Query<(
        &mut Character, 
        &CharacterSprites,
        &mut Handle<Image>
    ), (With<Character>, Without<Background>)>,
    mut event_emotion_change: EventReader<EmotionChangeEvent>,
    
    _text_object_query: Query<(&mut Text, &mut GUIScrollText)>,
    _scroll_stopwatch: ResMut<ChatScrollStopwatch>,
){
    for ev in event_emotion_change.iter() {
        for (mut character, sprites, mut current_sprite) in character_query.iter_mut() {
            if character.name == ev.name {
                character.emotion = ev.emotion.to_owned();
                *current_sprite = sprites.outfits.get(&character.outfit)
                    .expect("'{character.outfit}' attribute does not exist!")
                    .get(&character.emotion)
                    .expect("'default_emotion' atttribute does not exist!")
                    .clone();
                println!("[ Set emotion of '{}' to '{}']", ev.name, ev.emotion);
            }
        }
    }
}






/*
                           _ _
  ___ ___  _ __ ___  _ __ (_| | ___ _ __
 / __/ _ \| '_ ` _ \| '_ \| | |/ _ | '__|
| (_| (_) | | | | | | |_) | | |  __| |
 \___\___/|_| |_| |_| .__/|_|_|\___|_|
                    |_|
*/
mod ettethread_compiler;
use crate::ettethread_compiler::*;

/* Custom Types */
#[derive(Clone, Debug)]
pub enum Transition {
    Say(String, String),
    SetEmotion(String, String),
    SetBackground(String),
    SetGUI(String, String),
    GPTGet(String, String),
    GPTSay(String, String),
    Log(String),
    Scene(String),
    End
}
impl Transition {
    fn call(
        &self, 
        character_say_event: &mut EventWriter<CharacterSayEvent>,
        emotion_change_event: &mut EventWriter<EmotionChangeEvent>,
        background_change_event: &mut EventWriter<BackgroundChangeEvent>,
        gui_change_event: &mut EventWriter<GUIChangeEvent>,
        gpt_say_event: &mut EventWriter<GPTSayEvent>,
        gpt_get_event: &mut EventWriter<GPTGetEvent>,

        game_state: &mut ResMut<VisualNovelState>, 
    ) {
        match self {
            Transition::Say(character_name, msg) => {
                info!("Calling Transition::Say");
                character_say_event.send(CharacterSayEvent {
                    name: character_name.to_owned(),
                    message: msg.to_owned()
                });
                game_state.blocking = true;
            },
            Transition::SetEmotion(character_name, emotion) => {
                info!("Calling Transition::SetEmotion");
                emotion_change_event.send(EmotionChangeEvent {
                    name: character_name.to_owned(),
                    emotion: emotion.to_owned()
                });
            }
            Transition::SetBackground(background_id) => {
                info!("Calling Transition::SetBackground");
                background_change_event.send(BackgroundChangeEvent {
                    background_id: background_id.to_owned()
                });
            },
            Transition::SetGUI(gui_id, sprite_id) => {
                info!("Calling Transition::SetGUI");
                gui_change_event.send(GUIChangeEvent {
                    gui_id: gui_id.to_owned(),
                    sprite_id: sprite_id.to_owned()
                });
            },
            Transition::GPTSay(character_name, character_goal) => {
                info!("Calling Transition::GPTSay");
                game_state.blocking = true;
                gpt_say_event.send(GPTSayEvent {
                    name: character_name.to_owned(),
                    goal: character_goal.to_owned(),
                    advice: None
                });
            },
            Transition::GPTGet(past_character, past_goal) => {
                info!("Calling Transition::GPTGet");
                game_state.blocking = true;
                gpt_get_event.send(GPTGetEvent {past_character: past_character.clone(), past_goal: past_goal.clone()});
            },
            Transition::Log(msg) => println!("{msg}"),
            Transition::Scene(id) => {
                info!("Calling Transition::Scene");
                let script_transitions = game_state.all_script_transitions
                    .get(id.as_str())
                    .expect(&format!("Missing {id} script file! Please remember the game requires an entry.txt script file to have a starting position."))
                    .clone();
                game_state.transitions_iter = script_transitions.into_iter();
                game_state.current_scene_id = id.clone();
            },
            Transition::End => {
                todo!();
            }
        }
    }
}
pub struct Compiler;
impl Plugin for Compiler {
    fn build(&self, app: &mut App){
        app.add_startup_system(pre_compile)
            .add_system(run_transitions);
    }
}
fn pre_compile( mut game_state: ResMut<VisualNovelState>){
    info!("Starting pre-compilation");
    /* Character Setup */
    // Asset Gathering
    let mut all_script_transitions = HashMap::<String, Vec<Transition>>::new();
    let scripts_dir = std::env::current_dir()
        .expect("Failed to get current directory!")
        .join("assets")
        .join("scripts");
    let scripts_dir_entries: Vec<std::fs::DirEntry> = fs::read_dir(scripts_dir)
        .expect("Unable to read scripts folder!")
        .filter_map(|entry_result| {
            entry_result.ok()
        })
        .collect();
    for script_file_entry in scripts_dir_entries {
        let script_name: String = script_file_entry
            .path()
            .as_path()
            .file_stem().expect("Your script file must have a name! `.txt` is illegal.")
            .to_str().expect("Malformed UTF-8 in script file name, please verify it meets UTF-8 validity!")
            .to_owned();
        let script_contents: String = fs::read_to_string(script_file_entry.path())
            .expect("Contents of the script file must be valid UTF-8!");
        let script_transitions: Vec<Transition> = compile_to_transitions(script_contents);

        println!("[ [ Imported script '{}'! ] ]", script_name);
        all_script_transitions.insert(script_name, script_transitions);
    }

    // Setup entrypoint
    let entry = all_script_transitions
        .get("entry")
        .expect("Missing 'entry' script file! Please remember the game requires an entry.txt script file to have a starting position.")
        .clone();
    game_state.transitions_iter = entry.into_iter();
    game_state.all_script_transitions = all_script_transitions;
    game_state.current_scene_id = String::from("entry");

    game_state.blocking = false;

    info!("Completed pre-compilation");
}
fn run_transitions ( 
    mut character_say_event: EventWriter<CharacterSayEvent>,
    mut emotion_change_event: EventWriter<EmotionChangeEvent>,
    mut background_change_event: EventWriter<BackgroundChangeEvent>,
    mut gui_change_event: EventWriter<GUIChangeEvent>,
    mut gpt_say_event: EventWriter<GPTSayEvent>,
    mut gpt_get_event: EventWriter<GPTGetEvent>,

    mut game_state: ResMut<VisualNovelState>,
) {
    loop {
        while game_state.extra_transitions.len() > 0 {
            if game_state.blocking {
                return;
            }
            if let Some(transition) = game_state.extra_transitions.pop() {
                transition.call(
                    &mut character_say_event,
                    &mut emotion_change_event,
                    &mut background_change_event,
                    &mut gui_change_event,
                    &mut gpt_say_event,
                    &mut gpt_get_event,
    
                    &mut game_state,);
            }
        }
        if game_state.blocking {
            return;
        }
        match game_state.transitions_iter.next() {
            Some(transition) => {
                transition.call(
                    &mut character_say_event,
                    &mut emotion_change_event,
                    &mut background_change_event,
                    &mut gui_change_event,
                    &mut gpt_say_event,
                    &mut gpt_get_event,

                    &mut game_state,);
            },
            None => {
                return;
            }
        }
    }
}

/*
                 _
 _ __ ___   __ _(_)_ __
| '_ ` _ \ / _` | | '_ \
| | | | | | (_| | | | | |
|_| |_| |_|\__,_|_|_| |_|
*/
fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: String::from("Ettethread - Requiem"),
                    resolution: (1280., 800.).into(),
                    present_mode: PresentMode::AutoVsync,
                    // Tells wasm to resize the window according to the available canvas
                    fit_canvas_to_parent: true,
                    // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
                })
        ) 
        .init_resource::<VisualNovelState>()
        .add_startup_system(setup)
        .add_plugin(Compiler)
        .add_plugin(BackgroundController)
        .add_plugin(CharacterController)
        .add_plugin(ChatController)
        .run();
}

fn setup(
    mut commands: Commands,
    mut game_state: ResMut<VisualNovelState>,
) {
    /* Config */
    game_state.playername = String::from("Bolt");
    game_state.api_key = env::var("OPENAI_API_KEY").expect("Environment variable OPENAI_API_KEY needs to be set!");

    /* Basic Scene Setup */
    commands.spawn(Camera2dBundle::default());
}