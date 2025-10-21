use std::ops::Index;

use bevy::prelude::*;

use crate::{character::{controller::{FadingCharacters, SpriteKey}, CharacterConfig, CharactersResource}, Object, VisualNovelState};

pub fn change_character_emotion(
    sprite: &mut Sprite,
    sprites: &Res<CharactersResource>,
    emotion: &str,
    config: &CharacterConfig
) {
   let sprite_key = SpriteKey {
       character: config.name.clone(),
       outfit: config.outfit.clone(),
       emotion: emotion.to_owned()
   };
   let image = sprites.0.get(&sprite_key).expect(&format!("Sprite not found for {:?}", sprite_key));
   sprite.image = image.clone();
}
pub fn apply_alpha(
    mut commands: Commands,
    mut query: Query<&mut Sprite>,
    mut fading_characters: ResMut<FadingCharacters>,
    mut game_state: ResMut<VisualNovelState>,
) {
    if fading_characters.0.is_empty() {
        return;
    }

    let mut finished_anim: Vec<Entity> = Vec::new();
    for fading_char in &fading_characters.0 {
        let mut s = match query.get_mut(fading_char.0) {
            Ok(e) => e,
            Err(_) => continue
        };
        let mut color = s.color;
        color.set_alpha(s.color.alpha() + fading_char.1);
        s.color = color;
        if color.alpha() >= 1. || color.alpha() <= 0. {
            finished_anim.push(fading_char.0);
        }
    }
    let mut to_remove: Vec<usize> = Vec::new();
    let _ = fading_characters.0.iter().enumerate().for_each(|f| {
        if finished_anim.contains(&f.1.0) {
            to_remove.push(f.0);
        }
    });
    to_remove.reverse();
    for index in to_remove {
        let item = fading_characters.0.index(index);
        let to_despawn = item.2;
        if to_despawn {
            commands.entity(item.0).despawn();
        }
        fading_characters.0.remove(index);
    }
    if fading_characters.0.is_empty() {
        game_state.blocking = false;
    }
}
pub fn spawn_character(
    commands: &mut Commands,
    character_config: CharacterConfig,
    sprites: &Res<CharactersResource>,
    fading: &bool,
    fading_characters: &mut ResMut<FadingCharacters>,
) {
    let sprite_key = SpriteKey {
        character: character_config.name.clone(),
        outfit: character_config.outfit.clone(),
        emotion: character_config.emotion.clone(),
    };
    let image = match sprites.0.get(&sprite_key) {
        Some(s) => s.clone(),
        None => {
            eprintln!("No sprite found for {:?}", sprite_key);
            return;
        }
    };
    let entity = commands.spawn((
        Object {
            id: format!("_character_{}", character_config.name),
        },
        Sprite {
            image,
            color: Color::default().with_alpha(if *fading {
                0.
            } else { 1. }),
            ..default()
        },
        Transform::default()
            .with_translation(Vec3 {
                x: 0.,
                y: -40.,
                z: 1.,
            })
            .with_scale(Vec3 {
                x: 0.75,
                y: 0.75,
                z: 1.,
            }),
        character_config
    )).id();
    if *fading {
        fading_characters.0.push((entity, 0.01, false));
    }
}