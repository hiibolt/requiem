use bevy::{
    prelude::*,
    window::*,
    asset::{ Handle }
};
use std::fs;
use std::vec::IntoIter;
use std::collections::HashMap;
use regex::Regex;
use json::parse;


#[derive(Resource, Default)]
struct VisualNovelState {
    //backgrounds: HashMap<String, image::Handle>,

    transitions_iter: IntoIter<Transition>,
    blocking: bool,
    current_background: String,
}

#[derive(Component)]
struct Character {
    name: String,
    outfit: String,
    emotion: String,
    description: String,
    emotions: Vec<String>,
    xpos: f32,
    ypos: f32,
    scale: f32,
    opacity: f32
}
#[derive(Component)]
struct CharacterSprites {
    outfits: HashMap::<String, HashMap<String, Handle<Image>>>,
}
#[derive(Resource)]
struct OpacityFadeTimer(Timer);

pub struct CharacterController;
impl Plugin for CharacterController {
    fn build(&self, app: &mut App){
        app.insert_resource(OpacityFadeTimer(Timer::from_seconds(0.005, TimerMode::Repeating)))
            .add_startup_system(import_characters)
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
        .filter_map(|entry| {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                Some(entry.path())
            } else {
                None
            }
        });
    for outfit_dir in outfit_dirs {
        let mut emotion_sprites = HashMap::<String, Handle<Image>>::new();
        let outfit_name = outfit_dir
            .file_name().unwrap()
            .to_str().unwrap()
            .to_string();
        
        let sprite_paths = fs::read_dir(outfit_dir)
            .expect("No character data!")
            .map(|entry| entry.unwrap().path());
        for sprite_path in sprite_paths {
            let sprite_name = sprite_path
                .file_name().unwrap()
                .to_str().unwrap()
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

    commands.spawn((
    Character {
        name: parsed_character["name"].as_str().expect("Missing 'name' attribute").to_owned(),
        outfit: parsed_character["default_outfit"].as_str().expect("Missing 'name' attribute").to_owned(),
        emotion: parsed_character["default_emotion"].as_str().expect("Missing 'name' attribute").to_owned(),
        description: parsed_character["description"].as_str().expect("Missing 'name' attribute").to_owned(),
        emotions: parsed_character["emotions"]
            .members()
            .map(|entry| entry.as_str()
                .expect("Missing 'name' attribute")
                .to_owned()
            ).collect::<Vec<String>>(),
        xpos: 0.,
        ypos: -40.,
        scale: 0.75,
        opacity: 0.
    },
    CharacterSprites { outfits },
    SpriteBundle {
        ..default()
    }));
}
fn update_characters(
    mut query: Query<(
        &mut Character, 
        &CharacterSprites, 
        &mut Transform, 
        &mut Handle<Image>, 
        &mut Sprite
    )>,
    time: Res<Time>,
    mut timer: ResMut<OpacityFadeTimer>
){
    for (mut character, sprites, mut transform, mut current_sprite, mut sprite) in query.iter_mut() {
        // Fade the character in
        if character.opacity < 1. && timer.0.tick(time.delta()).just_finished() {
            character.opacity += 0.02;
        }

        // Update positioning, sprite, and opacity
        *transform = Transform::IDENTITY
            .with_translation(Vec3 { x:character.xpos, y:character.ypos, z:0. } )
            .with_scale(Vec3 { x:character.scale, y:character.scale, z:1. } );
        *current_sprite = sprites.outfits.get(&character.outfit)
            .expect("'{character.outfit}' attribute does not exist!")
            .get(&character.emotion)
            .expect("'default_emotion' atttribute does not exist!")
            .clone();
        let _ = *sprite.color.set_a(character.opacity);
    }
}



enum Transition {
    Background(String),
    Say(String, String),
    Log(String),
    End
}
impl Transition {
    fn call(&self, game_state: &mut ResMut<VisualNovelState> ) {
        match self {
            Transition::Background(id) => {
                (*game_state).current_background = id.clone();
                println!("Set current background to '{id}'");
            },
            Transition::Say(_character_name, _msg) => {
                todo!();
            },
            Transition::Log(msg) => println!("{msg}"),
            Transition::End => println!("TODO: END")
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
    /* PRECOMPILATION */
    let command_structure = Regex::new(r"(\w+)(?: (\w+)\=`(.+?)`)+").unwrap();

    // Compile Script into a vector Transitions, then create an iterator over them
    let full_script_string: String = fs::read_to_string(std::env::current_dir()
            .expect("Failed to get current directory!")
            .join("assets")
            .join("scripts")
            .join("script.txt"))
        .expect("Issue reading file!");
    let transitions: Vec<Transition> = full_script_string.lines().map(move |line| {
        println!("[ Compiling ] `{line}`");

        let mut command_options: HashMap<String, String> = HashMap::new();

        // Grabs the command in its normal habitat
        let command_captures = command_structure.captures(line)
            .expect("Bad command structure! Ex: [cmd option1=`` option2=``]");

        // Remove the command identifier seperately
        let mut args = command_captures.iter();
        let cmd_id = args
            .nth(1)
            .expect("There should be a command definition.")
            .expect("There should be a match on the first.")
            .as_str();
        println!("CMD: `{cmd_id}`");

        // Adds each option from the command to the options hashmap
        while let Some(capture) = args.next() {
            let option: String = capture.map_or(String::from(""), |m| m.as_str().to_owned());
            let value: String  = args.next().expect("Missing value!").map_or(String::from(""), |m| m.as_str().to_owned());
            
            command_options.insert(option, value);
        }

        // Try to run the command
        match cmd_id {
            "log" => {
                let msg = command_options.get("msg")
                    .expect("Missing 'msg' option!")
                    .to_owned();
                return Transition::Log(msg);
            },
            "bg" => {
                let background_id = command_options.get("background")
                    .expect("Missing 'background' option!")
                    .to_owned();
                return Transition::Background(background_id);
            },
            "say" => {
                let character_id = command_options.get("character")
                    .expect("Missing 'character' option!")
                    .to_owned();
                let msg = command_options.get("msg")
                    .expect("Missing 'msg' option!")
                    .to_owned();
                return Transition::Say(character_id, msg);
            },
            "end" => {
                return Transition::End;
            }
            _ => panic!("Bad command! {cmd_id}")
        }
    }).collect();

    game_state.transitions_iter = transitions.into_iter();
    game_state.blocking = false;

    println!("[ Completed Compilation ]");
}
fn run_transitions ( mut game_state: ResMut<VisualNovelState> ) {
    loop {
        if game_state.blocking {
            return;
        }
        match game_state.transitions_iter.next() {
            Some(transition) => {
                transition.call(&mut game_state);
            },
            None => {
                return;
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: String::from("I am a window!"),
                resolution: (1200., 800.).into(),
                present_mode: PresentMode::AutoVsync,
                // Tells wasm to resize the window according to the available canvas
                fit_canvas_to_parent: true,
                // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        })) 
        .init_resource::<VisualNovelState>()
        .add_startup_system(setup)
        .add_plugin(Compiler)
        .add_plugin(CharacterController)
        .run();
}

fn setup(mut commands: Commands) {
    /* Basic Scene Setup */
    commands.spawn(Camera2dBundle::default());
}