use std::collections::HashMap;

use bevy::prelude::*;
use bevy::{app::{App, Plugin}, asset::{AssetServer, Handle}};

use crate::{Character, Object};

/* Components */
#[derive(Component)]
pub struct Background {
    pub background_sprites: HashMap::<String, Handle<Image>>
}

/* Events */
#[derive(Event)]
pub struct BackgroundChangeEvent {
    pub background_id: String
}

pub struct BackgroundController;
impl Plugin for BackgroundController {
    fn build(&self, app: &mut App) {
        app.add_event::<BackgroundChangeEvent>()
            .add_systems(Startup, import_backgrounds)
            .add_systems(Update, update_background);
    }
}
pub fn import_backgrounds(mut commands: Commands, asset_server: Res<AssetServer>){
    let mut background_sprites: HashMap<String, Handle<Image>>= HashMap::new();
 
    let master_backgrounds_dir = std::env::current_dir()
        .expect("Failed to get current directory!")
        .join("assets")
        .join("backgrounds");
    let background_paths = std::fs::read_dir(master_backgrounds_dir)
        .expect("Unable to read outfit folders!")
        .map(|entry| entry.expect("Unable to read {entry}, IO error!").path());
    for background_path in background_paths {
        let background_name = background_path
            .file_stem().expect("Must have a complete background file name!")
            .to_str().expect("Invalid Unicode! Ensure your background file names are UTF-8!")
            .to_string();
        let background_texture = asset_server.load(background_path);

        println!("Imported background '{}'", background_name);
        background_sprites.insert(background_name, background_texture);
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
        SpriteBundle {
            transform: Transform::IDENTITY,
            ..default()
        }
    ));
}
pub fn update_background(
    mut background_query: Query<(
        &Background, 
        &mut Handle<Image>
    ), (With<Background>, Without<Character>)>,

    mut background_change_event: EventReader<BackgroundChangeEvent>,
){
    for ev in background_change_event.read() {
        for (background_obj, mut current_sprite) in background_query.iter_mut() {
            *current_sprite = background_obj.background_sprites.get(&ev.background_id)
                .expect("'{character.outfit}' attribute does not exist!")
                .clone();
            println!("[ Set background to '{}']", ev.background_id);
        }
    }
}