mod background;
mod character;
mod chat;
mod ettethread_ai;

use crate::background::*;
use crate::character::*;
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