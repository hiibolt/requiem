use std::collections::HashMap;

use bevy::prelude::*;
use json::parse;

use crate::{Background, ChatScrollStopwatch, GUIScrollText, Object};

/* Components */
#[derive(Component)]
pub struct Character {
    pub name: String,
    pub outfit: String,
    pub emotion: String,
    pub description: String,
    pub emotions: Vec<String>
}
#[derive(Component)]
pub struct CharacterSprites {
    pub outfits: HashMap::<String, HashMap<String, Handle<Image>>>,
}

/* Resources */
#[derive(Resource)]
pub struct OpacityFadeTimer(Timer);

/* Events */
#[derive(Event)]
pub struct EmotionChangeEvent {
    pub name: String,
    pub emotion: String
}

pub struct CharacterController;
impl Plugin for CharacterController {
    fn build(&self, app: &mut App) {
        app.insert_resource(OpacityFadeTimer(Timer::from_seconds(
            0.005,
            TimerMode::Repeating,
        )))
        .add_event::<EmotionChangeEvent>()
        .add_systems(Startup, import_characters)
        .add_systems(Update, update_characters);
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
    let outfit_dirs = std::fs::read_dir(master_character_dir)
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
        
        let sprite_paths = std::fs::read_dir(outfit_dir)
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
    let character_string: String = std::fs::read_to_string(std::env::current_dir()
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