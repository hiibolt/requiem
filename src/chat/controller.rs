use crate::{compiler::controller::{Controller, ControllerReadyMessage, TriggerControllersMessage}, Object, VisualNovelState};

use std::collections::HashMap;

use anyhow::Context;
use bevy::{asset::{LoadState, LoadedFolder}, color::palettes::css::RED, prelude::*, sprite::Anchor, text::{LineBreak, TextBounds}, time::Stopwatch, window::PrimaryWindow};

/* Messages */
#[derive(Message)]
pub struct CharacterSayMessage {
    pub name: String,
    pub message: String
}
#[derive(Message)]
pub struct GUIChangeMessage {
    pub gui_id: String,
    pub sprite_id: String
}

/* States */
#[derive(States, Debug, Default, Clone, Copy, Hash, Eq, PartialEq)]
enum ChatControllerState {
    #[default]
    Loading,
    Idle,
    Running,
}

/* Components */
#[derive(Component)]
pub struct GUIScrollText {
    pub message: String
}

/* Resources */
#[derive(Resource)]
pub struct ChatScrollStopwatch(Stopwatch);
#[derive(Resource)]
struct HandleToGuiFolder(Handle<LoadedFolder>);

/* Custom types */
#[derive(Bundle)]
struct TextBundle {
    object: Object,
    scroll_text: GUIScrollText,
    text: Text2d,
    layout: TextLayout,
    font: TextFont,
    color: TextColor,
    anchor: Anchor,
    transform: Transform,
    bounds: TextBounds,
    visibility: Visibility,
}

impl TextBundle {
    fn new(object: Object, text: &str) -> Self {
        Self {
            object,
            scroll_text: GUIScrollText { message: text.to_string() },
            text: Text2d(text.into()),
            layout: TextLayout::default(),
            font: TextFont::default(),
            color: TextColor::WHITE,
            anchor: Anchor::TOP_LEFT,
            transform: Transform::default(),
            bounds: TextBounds::default(),
            visibility: Visibility::default()
        }
    }

    fn with_font(self, font: TextFont) -> Self {
        Self {
            font,
            ..self
        }
    }

    fn with_layout(self, layout: TextLayout) -> Self {
        Self {
            layout,
            ..self
        }
    }

    fn with_color(self, color: TextColor) -> Self {
        Self {
            color,
            ..self
        }
    }

    fn with_anchor(self, anchor: Anchor) -> Self {
        Self {
            anchor,
            ..self
        }
    }

    fn with_transform(self, transform: Transform) -> Self {
        Self {
            transform,
            ..self
        }
    }

    fn with_bounds(self, bounds: TextBounds) -> Self {
        Self {
            bounds,
            ..self
        }
    }

    fn with_visibility(self, visibility: Visibility) -> Self {
        Self {
            visibility,
            ..self
        }
    }
}

pub struct ChatController;
impl Plugin for ChatController {
    fn build(&self, app: &mut App){
        app.insert_resource(ChatScrollStopwatch(Stopwatch::new()))
            .init_state::<ChatControllerState>()
            .add_systems(OnEnter(ChatControllerState::Loading), import_gui_sprites)
            .add_systems(Update, setup.run_if(in_state(ChatControllerState::Loading)))
            .add_message::<CharacterSayMessage>()
            .add_message::<GUIChangeMessage>()
            .add_systems(Update, wait_trigger.run_if(in_state(ChatControllerState::Idle)))
            .add_systems(OnEnter(ChatControllerState::Running), spawn_chatbox)
            .add_systems(Update, (update_chatbox, update_gui).run_if(in_state(ChatControllerState::Running)));
    }
}
fn setup(
    asset_server: Res<AssetServer>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    folder_handle: Res<HandleToGuiFolder>,
    mut game_state: ResMut<VisualNovelState>,
    mut controller_state: ResMut<NextState<ChatControllerState>>,
    mut msg_writer: MessageWriter<ControllerReadyMessage>,
) -> Result<(), BevyError> {
    let mut gui_sprites = HashMap::<String, Handle<Image>>::new();
    if let Some(state) = asset_server.get_load_state(folder_handle.0.id()) {
        match state {
            LoadState::Loaded => {
                if let Some(loaded_folder) = loaded_folders.get(folder_handle.0.id()) {
                    for handle in &loaded_folder.handles {
                        let path = handle.path()
                            .context("Error retrieving gui path")?;
                        let filename = path.path().file_stem()
                            .context("GUI file has no name")?
                            .to_string_lossy()
                            .to_string();
                        gui_sprites.insert(filename, handle.clone().typed());
                    }
                }

                game_state.gui_sprites = gui_sprites;
                controller_state.set(ChatControllerState::Idle);
                msg_writer.write(ControllerReadyMessage(Controller::Chat));
            },
            LoadState::Failed(e) => {
                return Err(anyhow::anyhow!("Error loading GUI assets: {}", e.to_string()).into());
            }
            _ => {}
        }
    }
    Ok(())
}
fn import_gui_sprites(mut commands: Commands, asset_server: Res<AssetServer> ){
    let loaded_folder = asset_server.load_folder("gui");
    commands.insert_resource(HandleToGuiFolder(loaded_folder));
}
fn spawn_chatbox(mut commands: Commands, asset_server: Res<AssetServer>){
    // Spawn Backplate + Nameplate
    commands.spawn((
        Object {
            id: String::from("_textbox_background")
        },
        Visibility::Hidden,
        Sprite::default(),
        Transform::from_xyz(0., -275., 2.),
    ))
    .with_children(|parent| {
        parent.spawn((
            Object {
                id: String::from("_namebox_background")
            },
            Visibility::Inherited,
            Sprite::default(),
            Transform::from_xyz(-270., 105., 2.).with_scale( Vec3 { x: 0.75, y: 0.75, z: 2. } ),
        ));
        parent.spawn(
            TextBundle::new(
                Object {
                    id: String::from("_name_text")
                },
                "UNFILLED"
            )
            .with_font(TextFont {
                           font: asset_server.load("fonts/ALLER.ttf"),
                           font_size: 40.0,
                           ..default()
                       })
            .with_anchor(Anchor::TOP_LEFT)
            .with_transform(Transform::from_xyz(-305., 126., 3.))
        );
        parent.spawn(
            TextBundle::new(
                Object {
                    id: String::from("_message_text")
                },
                "UNFILLED"
            )
            .with_font(TextFont {
                           font: asset_server.load("fonts/BOLDITALIC.ttf"),
                           font_size: 27.0,
                           ..default()
                       })
            .with_anchor(Anchor::TOP_LEFT)
            .with_transform(Transform::from_xyz(-350., 62., 3.))
            .with_bounds(TextBounds { width: Some(700.), height: Some(107.) }));
    });

    commands.spawn(
        TextBundle::new(
            Object {
                id: String::from("_info_text")
            },
            "",
        )
        .with_font(TextFont {
                       font: asset_server.load("fonts/BOLD.ttf"),
                       font_size: 50.,
                       ..default()
                   })
        .with_anchor(Anchor::TOP_CENTER)
        .with_layout(TextLayout {
                         justify: Justify::Center,
                         linebreak: LineBreak::WordBoundary,
                     })
        .with_color(TextColor(Color::Srgba(RED)))
        .with_transform(Transform::from_xyz(0., 302., 3.))
        .with_visibility(Visibility::Visible)
        .with_bounds(TextBounds { width: Some(700.), height: None })
    );
}
fn update_chatbox(
    mut event_message: MessageReader<CharacterSayMessage>,
    mut visibility_query: Query<(&mut Visibility, &Object)>,
    mut text_object_query: Query<(&mut Text2d, &mut GUIScrollText, &Object)>,
    mut scroll_stopwatch: ResMut<ChatScrollStopwatch>,

    mut game_state: ResMut<VisualNovelState>,

    time: Res<Time>,
    window: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
) -> Result<(), BevyError> {
    /* QUICK USE VARIABLES */
    let mut name_text_option: Option<&mut Text2d> = None;
    let mut info_text_option: Option<&mut Text2d> = None;
    let mut message_text_option: Option<&mut Text2d> = None;
    let mut message_scroll_text_obj_option: Option<&mut GUIScrollText> = None;
    
    for (text_literal, scroll_text_obj, text_obj) in text_object_query.iter_mut() {
        match text_obj.id.as_str() {
            "_name_text" => name_text_option = Some(text_literal.into_inner()),
            "_info_text" => info_text_option = Some(text_literal.into_inner()),
            "_message_text" => {
                message_text_option = Some(text_literal.into_inner());
                message_scroll_text_obj_option = Some(scroll_text_obj.into_inner());
            },
            _ => {}
        }
    }
    
    let name_text = name_text_option
        .context("Missing GUI text object with ID '_name_text'")?;
    let info_text = info_text_option
        .context("Missing GUI text object with ID '_info_text'")?;
    let message_text = message_text_option
        .context("Missing GUI text object with ID '_message_text'")?;
    let message_scroll_text_obj = message_scroll_text_obj_option
        .context("Missing GUI scroll text object with ID '_message_text'")?;

    let mut textbox_visibility_option: Option<&mut Visibility> = None;
    for (visibility_literal, textbox_obj) in visibility_query.iter_mut() {
        match textbox_obj.id.as_str() {
            "_textbox_background" => textbox_visibility_option = Some(visibility_literal.into_inner()),
            _ => {}
        }
    }
    let textbox_visibility = textbox_visibility_option
        .context("Missing GUI object with ID '_textbox_background'")?;

    // Tick clock
    let to_tick = if time.delta_secs() > 1. { std::time::Duration::from_secs_f32(0.) } else { time.delta() };
    scroll_stopwatch.0.tick(to_tick);

    /* STANDARD SAY EVENTS INITIALIZATION [Transition::Say] */
    for ev in event_message.read() {
        game_state.blocking = true;

        // Make the parent textbox visible
        *textbox_visibility = Visibility::Visible;

        // Reset the scrolling timer
        scroll_stopwatch.0.set_elapsed(std::time::Duration::from_secs_f32(0.));

        // Update the name
        let name = if ev.name == "[_PLAYERNAME_]" { game_state.playername.clone() } else { ev.name.clone() };
        name_text.0 = name;

        println!("MESSAGE {}", ev.message);

        message_scroll_text_obj.message = ev.message.clone();
    }

    // If the textbox is hidden, ignore the next section dedicated to updating it
    if *textbox_visibility == Visibility::Hidden {
        return Ok(());
    }

    // Take the original string from the message object
    let mut original_string: String = message_scroll_text_obj.message.clone();

    // Get the section of the string according to the elapsed time
    let length: u32 = (scroll_stopwatch.0.elapsed_secs() * 50.) as u32;

    // Return the section and apply it to the text object
    original_string.truncate(length as usize);
    message_text.0 = original_string;

    let window = window.single()
        .context("Failed to query for primary window")?;
    
    if let Some(position) = window.cursor_position() {
        let resolution = &window.resolution;
        let textbox_bounds: [f32; 4] = [
            (resolution.width() / 2.) - (796. / 2.),
            (resolution.width() / 2.) + (796. / 2.),
            (resolution.height() / 2.) - (155. / 2.) + (275.),
            (resolution.height() / 2.) + (155. / 2.) + (275.),
        ];
        if ( position.x > textbox_bounds[0] && position.x < textbox_bounds[1] ) && ( position.y > textbox_bounds[2] && position.y < textbox_bounds[3] ) && buttons.just_pressed(MouseButton::Left) {
            if length < message_scroll_text_obj.message.len() as u32 {
                // Skip message scrolling
                scroll_stopwatch.0.set_elapsed(std::time::Duration::from_secs_f32(100000000.));
                return Ok(());
            }
            println!("[ Player finished message ]");
            info_text.0 = String::from("");

            // Hide textbox parent object
            *textbox_visibility = Visibility::Hidden;

            // Allow transitions to be run again
            game_state.blocking = false;
        }
    }
    
    Ok(())
}

fn wait_trigger(
    mut msg_reader: MessageReader<TriggerControllersMessage>,
    mut controller_state: ResMut<NextState<ChatControllerState>>,
) {
    if msg_reader.read().count() > 0 {
        controller_state.set(ChatControllerState::Running);
    }
}
fn update_gui(
    mut change_messages: MessageReader<GUIChangeMessage>,
    mut gui_query: Query<(&Object, &mut Sprite)>,

    game_state: Res<VisualNovelState>
) -> Result<(), BevyError> {
    for ev in change_messages.read() {
        for (gui_obj, mut current_sprite) in gui_query.iter_mut() {
            if gui_obj.id == ev.gui_id {
                let gui_sprite = game_state.gui_sprites.get(&ev.sprite_id)
                    .with_context(|| format!("GUI asset '{}' does not exist", ev.sprite_id))?;
                current_sprite.image = gui_sprite.clone();
                println!("[ Set GUI asset '{}' to '{}']", ev.gui_id, ev.sprite_id);
            }
        }
    }
    Ok(())
}
