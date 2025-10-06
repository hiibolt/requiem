use std::collections::HashMap;

use bevy::{asset::{LoadState, LoadedFolder}, prelude::*};
use serde::Deserialize;

use crate::{compiler::controller::{Controller, ControllerReadyMessage, TriggerControllersMessage}, Background, ChatScrollStopwatch, GUIScrollText, Object};

/* States */
#[derive(States, Debug, Default, Clone, Copy, Hash, Eq, PartialEq)]
enum CharacterControllerState {
    #[default]
    Loading,
    Idle,
    Running,
}
/* Components */
#[derive(Component, Debug, Default, Asset, TypePath, Deserialize, Clone)]
pub struct Character {
    pub name: String,
    pub outfit: Option<String>,
    pub emotion: Option<String>,
    pub description: String,
    pub emotions: Vec<String>,
    pub outfits: Vec<String>,
}
impl Character {
    fn to_show(&self) -> bool {
        self.outfit.is_some() && self.emotion.is_some()
    }
}
#[derive(Component, Debug)]
pub struct CharacterSprites {
    pub outfits: HashMap::<String, HashMap<String, Handle<Image>>>,
}

/* Resources */
#[derive(Resource)]
pub struct OpacityFadeTimer(Timer);
#[derive(Resource)]
struct HandleToCharactersFolder(Handle<LoadedFolder>);
#[derive(Resource)]
struct CharacterToAssets(HashMap<String, CharacterSprites>);

/* Messages */
#[derive(Message)]
pub struct EmotionChangeMessage {
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
        .add_message::<EmotionChangeMessage>()
        .init_state::<CharacterControllerState>()
        .add_systems(OnEnter(CharacterControllerState::Loading), import_characters)
        .add_systems(Update, setup.run_if(in_state(CharacterControllerState::Loading)))
        .add_systems(Update, wait_trigger.run_if(in_state(CharacterControllerState::Idle)))
        .add_systems(OnEnter(CharacterControllerState::Running), spawn_characters)
        .add_systems(Update, update_characters.run_if(in_state(CharacterControllerState::Running)));
    }
}
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    folder_handle: Res<HandleToCharactersFolder>,
    mut controller_state: ResMut<NextState<CharacterControllerState>>,
    mut ev_writer: MessageWriter<ControllerReadyMessage>,
) {
    if let Some(state) = asset_server.get_load_state(folder_handle.0.id()) {
        match state {
            LoadState::Loaded => {
                if let Some(loaded_folder) = loaded_folders.get(folder_handle.0.id()) {
                    let mut characters: HashMap<String, CharacterSprites> = HashMap::new();
                    for handle in &loaded_folder.handles {
                        let path = handle.path().expect("Error retrieving character asset path").path();
                        let name: String = match path.iter().nth(1).map(|s| s.to_string_lossy().into()) {
                            Some(name) => name,
                            None => continue
                        };
                        match path.iter().count() {
                            3 => {
                                characters
                                    .entry(name.clone())
                                    .or_insert_with(|| CharacterSprites {
                                        outfits: HashMap::new(),
                                    });
                            },
                            4 => {
                                let outfit = match path.iter().nth(2).map(|s| s.to_string_lossy().into()) {
                                    Some(outfit) => outfit,
                                    None => continue
                                };
                                let emotion = match path.iter().nth(3) {
                                    Some(os_str) => {
                                        let file = std::path::Path::new(os_str);
                                        let name = file.file_stem()
                                            .map(|s| s.to_string_lossy().into_owned());
                                        if let Some(n) = name {
                                            n
                                        } else { continue }
                                    },
                                    None => continue
                                };
                                let character_entry = characters
                                    .entry(name.clone())
                                    .or_insert_with(|| CharacterSprites {
                                        outfits: HashMap::new(),
                                    });

                                let outfit_entry = character_entry
                                    .outfits
                                    .entry(outfit)
                                    .or_insert_with(HashMap::new);

                                outfit_entry.insert(emotion, handle.clone().typed());
                            }
                            _ => {}
                        }
                    }
                    commands.insert_resource(CharacterToAssets(characters));
                    ev_writer.write(ControllerReadyMessage(Controller::Character));
                }
                controller_state.set(CharacterControllerState::Idle);
            },
            LoadState::Failed(e) => {
                panic!("Error loading assets... {}", e.to_string());
            }
            _ => {}
        }
    }
}
fn import_characters(mut commands: Commands, asset_server: Res<AssetServer>){
    let loaded_folder = asset_server.load_folder("characters");
    commands.insert_resource(HandleToCharactersFolder(loaded_folder));
}
fn wait_trigger(
    mut msg_reader: MessageReader<TriggerControllersMessage>,
    mut controller_state: ResMut<NextState<CharacterControllerState>>,
) {
    if msg_reader.read().count() > 0 {
        controller_state.set(CharacterControllerState::Running);
    }
}
fn spawn_characters(
    mut commands: Commands,
    characters: Res<Assets<Character>>,
    characters_map: Res<CharacterToAssets>,
) {
    let characters_to_show: HashMap<String, Character> = characters.iter().filter_map(|(_, c)| {
        if c.to_show() {
            Some((c.name.clone(), c.clone()))
        } else { None }
    }).collect();
    for (name, sprites) in &characters_map.0 {
        let character = match characters_to_show.get(name) {
            Some(c) => c,
            None => continue
        };
        let current_sprite = sprites.outfits.get(&character.outfit.clone().unwrap()).expect("character.outfit attribute does not exist!")
            .get(&character.emotion.clone().unwrap()).expect("character.emotion attribute does not exist!")
            .clone();
        commands.spawn((
            Object {
                r#type: String::from("character"),
                id: format!("_character_{}", character.name)
            },
            character.clone(),
            Sprite {
                image: current_sprite,
                ..default()
            },
            Transform::default()
                .with_translation(Vec3 { x:0., y:-40., z:1. } )
                .with_scale(Vec3 { x:0.75, y:0.75, z:1. } ),
            CharacterSprites { outfits: sprites.outfits.clone() }
        ));
    }
}
fn update_characters(
    mut character_query: Query<(
        &mut Character,
        &CharacterSprites,
        &mut Sprite
    ), (With<Character>, Without<Background>)>,
    mut message_emotion_change: MessageReader<EmotionChangeMessage>,

    _text_object_query: Query<(&mut Text, &mut GUIScrollText)>,
    _scroll_stopwatch: ResMut<ChatScrollStopwatch>,
){
    for msg in message_emotion_change.read() {
        for (mut character, sprites, mut current_sprite) in character_query.iter_mut() {
            if character.name == msg.name {
                character.emotion = Some(msg.emotion.to_owned());
                let outfit = match &character.outfit {
                    Some(outfit) => outfit,
                    None => {
                        println!("'character.outfit' attribute does not exist!");
                        continue;
                    }
                };
                let emotion = match &character.emotion {
                    Some(emotion) => emotion,
                    None => {
                        println!("'character.emotion' attribute does not exist!");
                        continue;
                    }
                };
                current_sprite.image = sprites.outfits.get(outfit)
                    .expect("'{outfit}' attribute does not exist!")
                    .get(emotion)
                    .expect("'default_emotion' atttribute does not exist!")
                    .clone();
                println!("[ Set emotion of '{}' to '{}']", msg.name, msg.emotion);
            }
        }
    }
}
