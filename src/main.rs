mod background;
mod character;
mod chat;
mod intelligence;
mod compiler;

use crate::background::*;
use crate::character::*;
use crate::chat::*;
use crate::intelligence::*;
use crate::compiler::*;

use bevy::{
    prelude::*,
    window::*,
    asset::Handle,
};
use std::vec::IntoIter;
use std::collections::HashMap;


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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: String::from("Ettethread - Requiem"),
                    resolution: (1280., 800.).into(),
                    present_mode: PresentMode::AutoVsync,
                    // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
                })
        )
        .init_resource::<VisualNovelState>()
        .add_systems(Startup, setup)
        .add_plugins((
            Compiler,
            BackgroundController,
            CharacterController,
            ChatController,
        ))
        .run();
}

fn setup(
    mut commands: Commands,
    mut game_state: ResMut<VisualNovelState>,
) {
    // These are constants which would normally
    //  be filled in by the player
    game_state.playername = String::from("Bolt");
    game_state.api_key = std::env::var("OPENAI_API_KEY").expect("Environment variable OPENAI_API_KEY needs to be set!");

    // Create our primary camera (which is
    //  necessary even for 2D games)
    commands.spawn(Camera2dBundle::default());
}