use bevy::{
    prelude::*,
    window::*,
};
use std::fs;
use std::path::Path;
use std::vec::IntoIter;
use std::collections::HashMap;
use regex::Regex;
use json::parse;

#[derive(Component)]
struct Character {
    name: String,
    outfit: String,
    emotion: String,
    description: String,
    emotions: Vec<String>,
    xpos: i32,
    ypos: i32,
    scale: f32,
    opacity: f32
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

#[derive(Resource, Default)]
struct VisualNovelState {
    //backgrounds: HashMap<String, image::Handle>,

    transitions_iter: IntoIter<Transition>,
    blocking: bool,
    current_background: String,
}

fn main() {
    /* WARM UP ASSETS */
    /*
    // Literal Asset Hashmaps
    let mut backgrounds = HashMap::new();
    let backgrounds_dir = std::env::current_dir()
        .expect("Failed to get current directory!")
        .join("assets")
        .join("backgrounds");
    let background_paths = fs::read_dir(backgrounds_dir)
        .expect("No backgrounds dir!")
        .map(|entry| entry.unwrap().path());
    for background_path in background_paths {
        let file_name = background_path
            .file_name().unwrap()
            .to_str().unwrap()
            .to_string();
        let file_texture = image::Handle::from_path(background_path);

        println!("Imported background '{}'", file_name);
        backgrounds.insert(file_name, file_texture);
    }
    */
    let character_string: String = fs::read_to_string(std::env::current_dir()
            .expect("Failed to get current directory!")
            .join("assets")
            .join("characters")
            .join("Kiyomi")
            .join("character.json"))
        .expect("Issue reading file!");
    let parsed = json::parse(&character_string).expect("Malformed JSON!");
    println!("{}", parsed["name"]);

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
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(SpriteBundle {
        texture: asset_server.load(Path::new("characters/Nayu/uniform_neutral/NEUTRAL.png")),
        transform: Transform::from_scale(Vec3 {x:0.5, y:0.5, z:1.}),
        ..default()
    });
}