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
    gui_sprites: HashMap<String, Handle<Image>>,
    transitions_iter: IntoIter<Transition>,
    blocking: bool
}

#[derive(Component)]
struct Character {
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
#[derive(Resource)]
struct OpacityFadeTimer(Timer);

#[derive(Component)]
struct Background {
    background_sprites: HashMap::<String, Handle<Image>>
}

pub struct BackgroundController;
impl Plugin for BackgroundController {
    fn build(&self, app: &mut App){
        app.add_startup_system(import_backgrounds);
    }
}
fn import_backgrounds(mut commands: Commands, asset_server: Res<AssetServer>){
    let mut background_sprites: HashMap<String, Handle<Image>>= HashMap::new();

    let master_backgrounds_dir = std::env::current_dir()
        .expect("Failed to get current directory!")
        .join("assets")
        .join("backgrounds");
    let background_paths = fs::read_dir(master_backgrounds_dir)
        .expect("Unable to read outfit folders!")
        .map(|entry| entry.unwrap().path());
    for background_path in background_paths {
        let background_name = background_path
            .file_stem().expect("Must have a complete file name!")
            .to_str().unwrap()
            .to_string();
        let background_texture = asset_server.load(background_path);

        println!("Imported background '{}'", background_name);
        background_sprites.insert(background_name, background_texture);
    }

    /* Background Setup */
    commands.spawn((
        Background {
            background_sprites,
        }, 
        SpriteBundle {
            transform: Transform::IDENTITY,
            ..default()
        }
    ));
}


#[derive(Component)]
struct GUISprite {
    id: String,
}

#[derive(Component)]
struct GUIScrollText {
    id: String,
}

pub struct ChatController;
impl Plugin for ChatController {
    fn build(&self, app: &mut App){
        app.add_startup_system(import_gui_sprites)
            .add_startup_system(spawn_chatbox);
    }
}
fn import_gui_sprites( mut game_state: ResMut<VisualNovelState>, asset_server: Res<AssetServer> ){
    let mut gui_sprites = HashMap::<String, Handle<Image>>::new();
    let master_gui_dir = std::env::current_dir()
        .expect("Failed to get current directory!")
        .join("assets")
        .join("gui");
    let gui_sprite_paths = fs::read_dir(master_gui_dir)
        .expect("Unable to read outfit folders!")
        .map(|entry| entry.unwrap().path());
    for gui_sprite_path in gui_sprite_paths {
        let file_name = gui_sprite_path
            .file_stem().unwrap()
            .to_str().unwrap()
            .to_string();
        println!("[ Importing GUI asset '{file_name}' ]");
        gui_sprites.insert(file_name, asset_server.load(gui_sprite_path));
    }
    game_state.gui_sprites = gui_sprites;
}
fn spawn_chatbox(mut commands: Commands, mut game_state: ResMut<VisualNovelState>, asset_server: Res<AssetServer>){
    // Spawn Backplate + Nameplate
    commands.spawn((
        GUISprite {
            id: String::from("textbox_background")
        },
        SpriteBundle {
            visibility: Visibility::Visible,
            transform: Transform::from_xyz(0., -275., 2.),
            ..default()
        }
    ))
    .with_children(|parent| {
        parent.spawn((
            GUISprite {
                id: String::from("namebox_background")
            },
            SpriteBundle {
                visibility: Visibility::Inherited,
                transform: Transform::from_xyz(-270., 105., 2.)
                    .with_scale( Vec3 { x: 0.75, y: 0.75, z: 2. } ),
                ..default()
            }
        ));
    });

    // Spawn Text
    commands.spawn((
        GUIScrollText {
            id: String::from("text_ui")
        },
        TextBundle::from_section(
            "Nayu_todo",
            TextStyle {
                font: asset_server.load("fonts/ALLER.ttf"),
                font_size: 100.0,
                color: Color::WHITE,
            },
        ).with_text_alignment(TextAlignment::Left)
            .with_style(Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    bottom: Val::Px(215.0),
                    left: Val::Px(335.0),
                    ..default()
                },
                ..default()
            })
    ));
}





pub struct CharacterController;
impl Plugin for CharacterController {
    fn build(&self, app: &mut App){
        app.insert_resource(OpacityFadeTimer(Timer::from_seconds(0.005, TimerMode::Repeating)))
            .add_startup_system(import_characters);
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
                .file_stem().unwrap()
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

    let name = parsed_character["name"].as_str().expect("Missing 'name' attribute").to_owned();
    let outfit = parsed_character["default_outfit"].as_str().expect("Missing 'name' attribute").to_owned();
    let emotion = parsed_character["default_emotion"].as_str().expect("Missing 'name' attribute").to_owned();
    commands.spawn((
    Character {
        name: name.clone(),
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
/*
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
        *current_sprite = sprites.outfits.get(&character.outfit)
            .expect("'{character.outfit}' attribute does not exist!")
            .get(&character.emotion)
            .expect("'default_emotion' atttribute does not exist!")
            .clone();
        //let _ = *sprite.color.set_a(character.opacity);
    }
}*/



enum Transition {
    Say(String, String),
    SetEmotion(String, String),
    SetBackground(String),
    SetGUI(String, String),
    Log(String),
    End
}
impl Transition {
    fn call(
        &self, 
        game_state: &mut ResMut<VisualNovelState>, 
        character_query: &mut Query<(
            &mut Character, 
            &CharacterSprites,
            &mut Handle<Image>
        ), (With<Character>, Without<Background>)>,
        background_query: &mut Query<(
            &Background, 
            &mut Handle<Image>
        ), (With<Background>, Without<Character>)>,
        gui_query: &mut Query<(
            &GUISprite, 
            &mut Handle<Image>
        ), (With<GUISprite>, Without<Character>, Without<Background>)>,
    ) {
        match self {
            Transition::Say(_character_name, _msg) => {
                todo!();
            },
            Transition::SetEmotion(character_name, emotion) => {
                for (mut character, sprites, mut current_sprite) in character_query.iter_mut() {
                    if character.name == *character_name {
                        character.emotion = emotion.to_owned();
                        *current_sprite = sprites.outfits.get(&character.outfit)
                            .expect("'{character.outfit}' attribute does not exist!")
                            .get(&character.emotion)
                            .expect("'default_emotion' atttribute does not exist!")
                            .clone();
                        println!("[ Set emotion of '{character_name}' to '{emotion}']");
                    }
                }
            }
            Transition::SetBackground(background_id) => {
                for (background_obj, mut current_sprite) in background_query.iter_mut() {
                    *current_sprite = background_obj.background_sprites.get(background_id)
                        .expect("'{character.outfit}' attribute does not exist!")
                        .clone();
                    println!("[ Set background to '{background_id}']");
                }
            },
            Transition::SetGUI(gui_id, sprite_id) => {
                for (gui_obj, mut current_sprite) in gui_query.iter_mut() {
                    if gui_obj.id == *gui_id {
                        *current_sprite = game_state.gui_sprites.get(sprite_id)
                            .expect("GUI asset '{sprite_id}' does not exist!")
                            .clone();
                        println!("[ Set GUI asset '{gui_id}' to '{sprite_id}']");
                    }
                }
            },
            Transition::Log(msg) => println!("{msg}"),
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
    /* PRECOMPILATION */
    let command_structure = Regex::new(r"(\w+)[\s$]").unwrap();
    let argument_structure = Regex::new(r"(\w+)=`([^`]*)`").unwrap();
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

        // Remove the command identifier seperately
        let cmd_id = command_structure.captures_iter(line)
            .next()
            .unwrap()
            .iter()
            .nth(1)
            .unwrap()
            .expect("??")
            .as_str();
        println!("CMD: `{cmd_id}`");

        
        // Adds each option from the command to the options hashmap
        let mut args = argument_structure.captures_iter(line);
        while let Some(capture) = args.next() {
            let mut argument = capture.iter();
            println!("Field - {}", argument.next().unwrap().unwrap().as_str());
            let option: String = argument.next().expect("Missing field!").map_or(String::from(""), |m| m.as_str().to_owned());
            let value: String  = argument.next().expect("Missing value!").map_or(String::from(""), |m| m.as_str().to_owned());
            
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
            "say" => {
                let character_id = command_options.get("character")
                    .expect("Missing 'character' option!")
                    .to_owned();
                let msg = command_options.get("msg")
                    .expect("Missing 'msg' option!")
                    .to_owned();
                return Transition::Say(character_id, msg);
            },
            "set" => {
                let type_of = command_options.get("type")
                    .expect("Missing 'type' option!")
                    .as_str();
                match type_of {
                    "emotion" => {
                        let character_name = command_options.get("character")
                            .expect("Missing 'character' option required for type 'emotion'!")
                            .to_owned();
                        let emotion = command_options.get("emotion")
                            .expect("Missing 'emotion' option required for type 'emotion'!")
                            .to_owned();
                        return Transition::SetEmotion(character_name, emotion);
                    },
                    "background" => {
                        let background_id = command_options.get("background")
                            .expect("Missing 'background' option required for type 'background'!")
                            .to_owned();
                        return Transition::SetBackground( background_id );
                    }
                    "GUI" => {
                        let gui_id = command_options.get("id")
                            .expect("Missing 'id' option required for type 'GUI'!")
                            .to_owned();
                        let sprite_id = command_options.get("sprite")
                            .expect("Missing 'sprite' option required for type 'GUI'!")
                            .to_owned();
                        return Transition::SetGUI( gui_id, sprite_id );
                    }
                    _ => panic!("Bad type '{type_of}'!")
                }
            }
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
fn run_transitions ( 
    mut game_state: ResMut<VisualNovelState>, 
    mut character_query: Query<(
        &mut Character, 
        &CharacterSprites,
        &mut Handle<Image>
    ), (With<Character>, Without<Background>)>,
    mut background_query: Query<(&Background, &mut Handle<Image>), (With<Background>, Without<Character>)>,
    mut gui_query: Query<(
        &GUISprite, 
        &mut Handle<Image>
    ), (With<GUISprite>, Without<Character>, Without<Background>)>
) {
    loop {
        if game_state.blocking {
            return;
        }
        match game_state.transitions_iter.next() {
            Some(transition) => {
                transition.call(&mut game_state, &mut character_query, &mut background_query, &mut gui_query);
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
        })) 
        .init_resource::<VisualNovelState>()
        .add_startup_system(setup)
        .add_plugin(Compiler)
        .add_plugin(BackgroundController)
        .add_plugin(CharacterController)
        .add_plugin(ChatController)
        .run();
}

fn setup(mut commands: Commands) {
    /* Basic Scene Setup */
    commands.spawn(Camera2dBundle::default());
}