use std::collections::HashMap;

use bevy::asset::{LoadState, LoadedFolder};
use bevy::prelude::*;
use bevy::{app::{App, Plugin}, asset::{AssetServer, Handle}};

use crate::compiler::controller::{Controller, ControllerReadyMessage, TriggerControllersMessage};
use crate::{Character, Object};

/* States */
#[derive(States, Debug, Default, Clone, Copy, Hash, Eq, PartialEq)]
enum BackgroundControllerState {
    #[default]
    Loading,
    Idle,
    Running,
}

/* Components */
#[derive(Component)]
pub struct Background {
    pub background_sprites: HashMap::<String, Handle<Image>>
}

/* Resources */
#[derive(Resource)]
struct HandleToBackgroundsFolder(Handle<LoadedFolder>);

/* Messages */
#[derive(Message)]
pub struct BackgroundChangeMessage {
    pub background_id: String
}

pub struct BackgroundController;
impl Plugin for BackgroundController {
    fn build(&self, app: &mut App) {
        app.add_message::<BackgroundChangeMessage>()
            .init_state::<BackgroundControllerState>()
            .add_systems(OnEnter(BackgroundControllerState::Loading), import_backgrounds)
            .add_systems(Update, setup.run_if(in_state(BackgroundControllerState::Loading)))
            .add_systems(Update, wait_trigger.run_if(in_state(BackgroundControllerState::Idle)))
            .add_systems(Update, update_background.run_if(in_state(BackgroundControllerState::Running)));
    }
}
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    folder_handle: Res<HandleToBackgroundsFolder>,
    mut controller_state: ResMut<NextState<BackgroundControllerState>>,
    mut msg_writer: MessageWriter<ControllerReadyMessage>,
) {
    let mut background_sprites: HashMap<String, Handle<Image>>= HashMap::new();

    if let Some(state) = asset_server.get_load_state(folder_handle.0.id()) {
        match state {
            LoadState::Loaded => {
                if let Some(loaded_folder) = loaded_folders.get(folder_handle.0.id()) {
                    for handle in &loaded_folder.handles {
                        let filename = handle.path().expect("Error retrieving background path")
                            .path().file_stem().unwrap().to_string_lossy().to_string();
                        background_sprites.insert(filename, handle.clone().typed());
                    }
                }

                /* Background Setup */
                commands.spawn((
                    Object {
                        r#type: String::from("background"),
                        id: String::from("_primary")
                    },
                    Background {
                        background_sprites,
                    },
                    Sprite::default()
                ));
                controller_state.set(BackgroundControllerState::Idle);
                msg_writer.write(ControllerReadyMessage(Controller::Background));
            },
            LoadState::Failed(e) => {
                panic!("Error loading assets... {}", e.to_string());
            }
            _ => {}
        }
    }
}
pub fn import_backgrounds(mut commands: Commands, asset_server: Res<AssetServer>){
    let loaded_folder = asset_server.load_folder("backgrounds");
    commands.insert_resource(HandleToBackgroundsFolder(loaded_folder));
}
fn wait_trigger(
    mut msg_reader: MessageReader<TriggerControllersMessage>,
    mut controller_state: ResMut<NextState<BackgroundControllerState>>,
) {
    if msg_reader.read().count() > 0 {
        controller_state.set(BackgroundControllerState::Running);
    }
}
pub fn update_background(
    mut background_query: Query<(
        &Background,
        &mut Sprite
    ), (With<Background>, Without<Character>)>,

    mut background_change_message: MessageReader<BackgroundChangeMessage>,
){
    for msg in background_change_message.read() {
        for (background_obj, mut current_sprite) in background_query.iter_mut() {
            current_sprite.image = background_obj.background_sprites.get(&msg.background_id)
                .expect("background does not exist!")
                .clone();
            println!("[ Set background to '{}']", msg.background_id);
        }
    }
}
