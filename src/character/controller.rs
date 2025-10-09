use std::collections::HashMap;

use anyhow::{Result, Context};
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
    pub outfit: String,
    pub emotion: String,
    pub description: String,
    pub emotions: Vec<String>,
    pub outfits: Vec<String>,
}
#[derive(Component, Debug)]
pub struct CharacterSprites {
    pub outfits: HashMap::<String, HashMap<String, Handle<Image>>>,
}

/* Resources */
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
        app.add_message::<EmotionChangeMessage>()
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
) -> Result<(), BevyError> {
    if let Some(state) = asset_server.get_load_state(folder_handle.0.id()) {
        match state {
            LoadState::Loaded => {
                if let Some(loaded_folder) = loaded_folders.get(folder_handle.0.id()) {
                    let mut characters: HashMap<String, CharacterSprites> = HashMap::new();
                    for handle in &loaded_folder.handles {
                        let path = handle.path()
                            .context("Error retrieving character asset path")?
                            .path();
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
                return Err(anyhow::anyhow!("Error loading character assets: {}", e.to_string()).into());
            }
            _ => {}
        }
    }
    Ok(())
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
) -> Result<(), BevyError> {
    let characters: HashMap<String, Character> = characters.iter()
        .map(|(_, c)| (c.name.clone(), c.clone()))
        .collect();
    for (name, sprites) in &characters_map.0 {
        let character = characters.get(name)
            .with_context(|| format!("Character asset '{name}' does not exist!"))?;
        let outfit = &character.outfit;
        let emotion = &character.emotion;
        let current_sprite = sprites.outfits.get(outfit)
            .with_context(|| format!("Outfit '{outfit}'  does not exist!"))?
            .get(emotion)
            .with_context(|| format!("Emotion '{emotion}' does not exist!"))?;

        commands.spawn((
            Object {
                id: format!("_character_{}", character.name)
            },
            character.clone(),
            Sprite {
                image: current_sprite.clone(),
                ..default()
            },
            Transform::default()
                .with_translation(Vec3 { x:0., y:-40., z:1. } )
                .with_scale(Vec3 { x:0.75, y:0.75, z:1. } ),
            CharacterSprites { outfits: sprites.outfits.clone() }
        ));
    }

    Ok(())
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
) -> Result<(), BevyError> {
    for msg in message_emotion_change.read() {
        for (mut character, sprites, mut current_sprite) in character_query.iter_mut() {
            if character.name == msg.name {
                character.emotion = msg.emotion.to_owned();
                let outfit = &character.outfit;
                let emotion = &character.emotion;
                
                current_sprite.image = sprites.outfits.get(outfit)
                    .with_context(|| format!("'{outfit}' outfit sprite does not exist!"))?
                    .get(emotion)
                    .with_context(|| format!("'{emotion}' emotion sprite does not exist!"))?
                    .clone();
                println!("[ Set emotion of '{}' to '{}']", msg.name, msg.emotion);
            }
        }
    }

    Ok(())
}
