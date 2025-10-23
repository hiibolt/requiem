use crate::{chat::ui_provider::{backplate_container, messagetext, namebox, nametext, textbox, top_section}, compiler::controller::{Controller, ControllerReadyMessage, TriggerControllersMessage, UiRoot}, Object, VisualNovelState};

use std::collections::HashMap;

use anyhow::Context;
use bevy::{asset::{LoadState, LoadedFolder}, color::palettes::css::RED, prelude::*, sprite::Anchor, text::{LineBreak, TextBounds}, time::Stopwatch, ui::{debug::print_ui_layout_tree, ui_layout_system, RelativeCursorPosition}, window::PrimaryWindow};

/* Messages */
#[derive(Message)]
pub struct CharacterSayMessage {
    pub name: String,
    pub message: String
}
#[derive(Message)]
pub struct GUIChangeMessage {
    pub gui_target: GuiChangeTarget,
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
#[derive(Component)]
pub struct TextBoxBackground;
#[derive(Component)]
pub struct NameBoxBackground;
#[derive(Component)]
pub struct NameText;
#[derive(Component)]
struct MessageText;
#[derive(Component)]
struct InfoText;

/* Resources */
#[derive(Resource)]
pub struct ChatScrollStopwatch(Stopwatch);
#[derive(Resource)]
struct HandleToGuiFolder(Handle<LoadedFolder>);
#[derive(Resource)]
struct GuiImages(HashMap<String, Handle<Image>>);

/* Custom types */
#[derive(Debug, Clone)]
pub enum GuiChangeTarget {
    TextBoxBackground,
    NameBoxBackground,
}

#[derive(Bundle)]
pub struct TextBundle {
    object: Object,
    scroll_text: GUIScrollText,
    text: Text2d,
    layout: TextLayout,
    font: TextFont,
    color: TextColor,
    bounds: TextBounds,
    visibility: Visibility,
}

impl TextBundle {
    pub fn new(object: Object, text: &str) -> Self {
        Self {
            object,
            scroll_text: GUIScrollText { message: text.to_string() },
            text: Text2d(text.into()),
            layout: TextLayout::default(),
            font: TextFont::default(),
            color: TextColor::WHITE,
            bounds: TextBounds::default(),
            visibility: Visibility::default()
        }
    }

    pub fn with_font(self, font: TextFont) -> Self {
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
            .add_systems(Update, (update_chatbox, handle_click, update_gui).run_if(in_state(ChatControllerState::Running)));
    }
}
fn handle_click(
    relative_cursor: Single<&RelativeCursorPosition>
) {
    info!("HANDLE CLICK {}", relative_cursor.cursor_over());
}
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    folder_handle: Res<HandleToGuiFolder>,
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

                commands.insert_resource(GuiImages(gui_sprites));
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
fn spawn_chatbox(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    ui_root: Single<Entity, With<UiRoot>>,
){
    // Spawn Backplate + Nameplate

    // Container
    let container = commands.spawn(backplate_container()).id();
    commands.entity(ui_root.entity()).add_child(container);
    
    // Top section: Nameplate flex container
    let top_section = commands.spawn(top_section()).id();
    commands.entity(container).add_child(top_section);
    
    // Namebox Node
    let namebox = commands.spawn(namebox()).id();
    commands.entity(top_section).add_child(namebox);
    
    // NameText
    let nametext = commands.spawn(nametext(&asset_server)).id();
    commands.entity(namebox).add_child(nametext);
    
    // Backplate Node
    let textbox_bg = commands.spawn(textbox()).id();
    commands.entity(container).add_child(textbox_bg);
    
    // MessageText
    let messagetext = commands.spawn(messagetext(&asset_server)).id();
    commands.entity(textbox_bg).add_child(messagetext);
    
    // commands.spawn((
    //     TextBundle::new(
    //         Object {
    //             id: String::from("_info_text")
    //         },
    //         "",
    //     )
    //     .with_font(TextFont {
    //                    font: asset_server.load("fonts/BOLD.ttf"),
    //                    font_size: 50.,
    //                    ..default()
    //                })
    //     .with_anchor(Anchor::TOP_CENTER)
    //     .with_layout(TextLayout {
    //                      justify: Justify::Center,
    //                      linebreak: LineBreak::WordBoundary,
    //                  })
    //     .with_color(TextColor(Color::Srgba(RED)))
    //     .with_transform(Transform::from_xyz(0., 302., 3.))
    //     .with_visibility(Visibility::Visible)
    //     .with_bounds(TextBounds { width: Some(700.), height: None }),
    //     InfoText
    // ));
}
fn update_chatbox(
    mut event_message: MessageReader<CharacterSayMessage>,
    textbox_bg_visibility: Single<&mut Visibility, With<TextBoxBackground>>,
    mut text_object_query: Query<(&mut Text2d, &mut GUIScrollText, &Object)>,
    mut scroll_stopwatch: ResMut<ChatScrollStopwatch>,
    mut game_state: ResMut<VisualNovelState>,
    time: Res<Time>,
    window: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
) -> Result<(), BevyError> {
    /* QUICK USE VARIABLES */
    let mut name_text_option: Option<&mut Text2d> = None;
    // let mut info_text_option: Option<&mut Text2d> = None;
    let mut message_text_option: Option<&mut Text2d> = None;
    let mut message_scroll_text_obj_option: Option<&mut GUIScrollText> = None;
    
    // for (text_literal, scroll_text_obj, text_obj) in text_object_query.iter_mut() {
    //     match text_obj.id.as_str() {
    //         "_name_text" => name_text_option = Some(text_literal.into_inner()),
    //         // "_info_text" => info_text_option = Some(text_literal.into_inner()),
    //         "_message_text" => {
    //             message_text_option = Some(text_literal.into_inner());
    //             message_scroll_text_obj_option = Some(scroll_text_obj.into_inner());
    //         },
    //         _ => {}
    //     }
    // }
    
    // let name_text = name_text_option
    //     .context("Missing GUI text object with ID '_name_text'")?;
    // // let info_text = info_text_option
    //     // .context("Missing GUI text object with ID '_info_text'")?;
    // let message_text = message_text_option
    //     .context("Missing GUI text object with ID '_message_text'")?;
    // let message_scroll_text_obj = message_scroll_text_obj_option
    //     .context("Missing GUI scroll text object with ID '_message_text'")?;

    // Tick clock
    let to_tick = if time.delta_secs() > 1. { std::time::Duration::from_secs_f32(0.) } else { time.delta() };
    scroll_stopwatch.0.tick(to_tick);
    let mut textbox_bg_visibility = textbox_bg_visibility.into_inner();

    /* STANDARD SAY EVENTS INITIALIZATION [Transition::Say] */
    for ev in event_message.read() {
        game_state.blocking = true;

        // Make the parent textbox visible
        *textbox_bg_visibility = Visibility::Visible;

        // Reset the scrolling timer
        scroll_stopwatch.0.set_elapsed(std::time::Duration::from_secs_f32(0.));

        // Update the name
        let name = if ev.name == "[_PLAYERNAME_]" { game_state.playername.clone() } else { ev.name.clone() };
        // name_text.0 = name;

        println!("MESSAGE {}", ev.message);

        // message_scroll_text_obj.message = ev.message.clone();
    }

    // If the textbox is hidden, ignore the next section dedicated to updating it
    if *textbox_bg_visibility == Visibility::Hidden {
        return Ok(());
    }

    // Take the original string from the message object
    // let mut original_string: String = message_scroll_text_obj.message.clone();

    // Get the section of the string according to the elapsed time
    let length: u32 = (scroll_stopwatch.0.elapsed_secs() * 50.) as u32;

    // Return the section and apply it to the text object
    // original_string.truncate(length as usize);
    // message_text.0 = original_string;

    // let window = window.single()
    //     .context("Failed to query for primary window")?;
    
    // if let Some(position) = window.cursor_position() {
    //     let resolution = &window.resolution;
    //     let textbox_bounds: [f32; 4] = [
    //         (resolution.width() / 2.) - (796. / 2.),
    //         (resolution.width() / 2.) + (796. / 2.),
    //         (resolution.height() / 2.) - (155. / 2.) + (275.),
    //         (resolution.height() / 2.) + (155. / 2.) + (275.),
    //     ];
    //     if ( position.x > textbox_bounds[0] && position.x < textbox_bounds[1] ) && ( position.y > textbox_bounds[2] && position.y < textbox_bounds[3] ) && buttons.just_pressed(MouseButton::Left) {
    //         if length < message_scroll_text_obj.message.len() as u32 {
    //             // Skip message scrolling
    //             scroll_stopwatch.0.set_elapsed(std::time::Duration::from_secs_f32(100000000.));
    //             return Ok(());
    //         }
    //         println!("[ Player finished message ]");
    //         // info_text.0 = String::from("");

    //         // Hide textbox parent object
    //         *textbox_bg_visibility = Visibility::Hidden;

    //         // Allow transitions to be run again
    //         game_state.blocking = false;
    //     }
    // }
    
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
    mut param_set: ParamSet<(
        Single<&mut ImageNode, With<TextBoxBackground>>,
        Single<&mut ImageNode, With<NameBoxBackground>>,
    )>,
    gui_images: Res<GuiImages>,
) -> Result<(), BevyError> {
    for ev in change_messages.read() {
        let image = gui_images.0.get(&ev.sprite_id)
            .with_context(|| format!("GUI asset '{}' does not exist", ev.sprite_id))?;
        let target = match ev.gui_target {
            GuiChangeTarget::TextBoxBackground => &mut param_set.p0().image,
            GuiChangeTarget::NameBoxBackground => &mut param_set.p1().image,
        };
        *target = image.clone();
        
    }
    Ok(())
}
