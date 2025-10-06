use crate::{compiler::controller::{Controller, ControllerReadyMessage, TriggerControllersMessage}, intelligence::*, Character, Object, Transition, VisualNovelState};

use std::collections::HashMap;

use bevy::{asset::{LoadState, LoadedFolder}, color::palettes::css::RED, input::keyboard::{Key, KeyboardInput}, prelude::*, sprite::Anchor, text::{LineBreak, TextBounds}, time::Stopwatch, window::PrimaryWindow};

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
pub struct TypeBox;

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
            .add_message::<GPTSayMessage>()
            .add_message::<GPTGetMessage>()
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
) {
    let mut gui_sprites = HashMap::<String, Handle<Image>>::new();
    if let Some(state) = asset_server.get_load_state(folder_handle.0.id()) {
        match state {
            LoadState::Loaded => {
                if let Some(loaded_folder) = loaded_folders.get(folder_handle.0.id()) {
                    for handle in &loaded_folder.handles {
                        let filename = handle.path().expect("Error retrieving gui path").path().file_stem().unwrap().to_string_lossy().to_string();
                        gui_sprites.insert(filename, handle.clone().typed());
                    }
                }

                game_state.gui_sprites = gui_sprites;
                controller_state.set(ChatControllerState::Idle);
                msg_writer.write(ControllerReadyMessage(Controller::Chat));
            },
            LoadState::Failed(e) => {
                panic!("Error loading assets... {}", e.to_string());
            }
            _ => {}
        }
    }
}
fn import_gui_sprites(mut commands: Commands, asset_server: Res<AssetServer> ){
    let loaded_folder = asset_server.load_folder("gui");
    commands.insert_resource(HandleToGuiFolder(loaded_folder));
}
fn spawn_chatbox(mut commands: Commands, asset_server: Res<AssetServer>){
    // Spawn Backplate + Nameplate
    commands.spawn((
        Object {
            r#type: String::from("gui"),
            id: String::from("_textbox_background")
        },
        Visibility::Hidden,
        Sprite::default(),
        Transform::from_xyz(0., -275., 2.),
    ))
    .with_children(|parent| {
        parent.spawn((
            Object {
                r#type: String::from("gui"),
                id: String::from("_namebox_background")
            },
            Visibility::Inherited,
            Sprite::default(),
            Transform::from_xyz(-270., 105., 2.).with_scale( Vec3 { x: 0.75, y: 0.75, z: 2. } ),
        ));
        parent.spawn(
            TextBundle::new(
                Object {
                    r#type: String::from("gui"),
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
                    r#type: String::from("gui"),
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

    // Spawn typebox
    commands.spawn((
        Object {
            r#type: String::from("gui"),
            id: String::from("_typebox_background")
        },
        Visibility::Hidden,
        Transform::from_xyz(0., -275., 2.),
        Sprite::default(),
        TypeBox
    ))
    .with_children(|parent| {
        parent.spawn(
            TextBundle::new(
                Object {
                    r#type: String::from("gui"),
                    id: String::from("_type_text")
                },
                "Start typing..."
            )
            .with_font(TextFont {
                           font: asset_server.load("fonts/BOLDITALIC.ttf"),
                           font_size: 27.0,
                           ..default()
                       })
            .with_anchor(Anchor::TOP_LEFT)
            .with_transform(Transform::from_xyz(-350., 62., 3.))
            .with_bounds(TextBounds { width: Some(700.), height: Some(20000.) })
        );
    });

    commands.spawn(
        TextBundle::new(
            Object {
                r#type: String::from("gui"),
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
    mut gpt_message: MessageReader<GPTSayMessage>,
    mut get_message: MessageReader<GPTGetMessage>,
    character_query: Query<&Character>,
    mut visibility_query: Query<(&mut Visibility, &Object)>,
    mut text_object_query: Query<(&mut Text2d, &mut GUIScrollText, &Object), Without<TypeBox>>,
    mut scroll_stopwatch: ResMut<ChatScrollStopwatch>,

    mut input_messages: MessageReader<KeyboardInput>,

    mut game_state: ResMut<VisualNovelState>,

    time: Res<Time>,
    window: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    /* QUICK FUNCTIONS */
    // Returns a reference to a character object by its name
    let find_character = |wanted_character_name: &String| -> Option<&Character> {
        for character in character_query.iter() {
            if character.name == *wanted_character_name {
                return Some(character);
            }
        }
        None
    };
    /* QUICK USE VARIABLES */
    // Reference to SPECIFICALLY the typing text display object.
    // I'm well aware this is bad practice, but Bevy makes it really hard
    // to avoid this without UNREAL levels of indent.
    // I'm a JS dev who follows Torvald's rules about indentation, cry.
    let mut name_text_option: Option<&mut Text2d> = None;
    let mut type_text_option: Option<&mut Text2d> = None;
    let mut info_text_option: Option<&mut Text2d> = None;
    let mut message_text_option: Option<&mut Text2d> = None;
    let mut message_scroll_text_obj_option: Option<&mut GUIScrollText> = None;
    for (text_literal, scroll_text_obj, text_obj) in text_object_query.iter_mut() {
        match text_obj.id.as_str() {
            "_name_text" => name_text_option = Some(text_literal.into_inner()),
            "_info_text" => info_text_option = Some(text_literal.into_inner()),
            "_type_text" => type_text_option = Some(text_literal.into_inner()),
            "_message_text" => {
                message_text_option = Some(text_literal.into_inner());
                message_scroll_text_obj_option = Some(scroll_text_obj.into_inner());
            },
            _ => {}
        }
    }
    let name_text = name_text_option.expect("MISSING GUISCROLLTEXT OBJECT WITH ID 'name_text'!");
    let type_text = type_text_option.expect("MISSING GUISCROLLTEXT OBJECT WITH ID 'type_text'!");
    let info_text = info_text_option.expect("MISSING GUISCROLLTEXT OBJECT WITH ID 'info_text'!");
    let message_text = message_text_option.expect("MISSING GUISCROLLTEXT OBJECT WITH ID 'message_text'!");
    let message_scroll_text_obj = message_scroll_text_obj_option.expect("MISSING GUISCROLLTEXT OBJECT WITH ID 'message_text'!");

    // Reference to SPECIFICALLY the typing text display object
    let mut typebox_visibility_option: Option<&mut Visibility> = None;
    let mut textbox_visibility_option: Option<&mut Visibility> = None;
    for (visibility_literal, typebox_obj) in visibility_query.iter_mut() {
        match typebox_obj.id.as_str() {
            "_typebox_background" => typebox_visibility_option = Some(visibility_literal.into_inner()),
            "_textbox_background" => textbox_visibility_option = Some(visibility_literal.into_inner()),
            _ => {}
        }
    }
    let typebox_visibility = typebox_visibility_option.expect("MISSING GUI OBJECT WITH ID '_typebox_background'!");
    let textbox_visibility = textbox_visibility_option.expect("MISSING GUI OBJECT WITH ID '_textbox_background'!");

    // Tick clock (must be after everything)
    // basically if there's enough of a jump, it's not worth the stutters, preserve gameplay over ego :<
    let to_tick = if time.delta_secs() > 1. { std::time::Duration::from_secs_f32(0.) } else { time.delta() };
    scroll_stopwatch.0.tick(to_tick);

    /* GPT EVENTS [Transition::GPTSay] */
    for ev in gpt_message.read() {
        // Grab the character by reference matching the one notated in the event
        let character: &Character = find_character(&ev.name).expect("Couldn't find associated character!");

        /* GPT GENERATE - GENERATE MESSAGES FOR PLAYER INTERACTION */
        // Builds a full chat transition with the intent of completing a set goal
        // (contained in the event)
        let ret = generate_chat_transitions(&character, &game_state, &ev);
        match ret {
            Ok(mut transitions) => {
                println!("[ Inserting {} transitions... ]", transitions.len());
                game_state.extra_transitions.append(&mut transitions);

                /* GPT CHECK - CHECK IF THE CHARACTER ACHIEVED THEIR GOAL! */
                // Any errors here should literally result in continuation of the game.
                // It's way easier to let the next prompt be generated,
                // and let the player see the dialog that was just generated anyway
                // (plus, it could be OpenAI rate limits)
                if let Some(goal_status) = determine_goal_status(&character, &game_state, &ev){
                    println!("[ Goal Status: {} ]", goal_status);
                    if !goal_status {
                        println!("[ Inserting GPTGet transition... ]");
                        game_state.extra_transitions.insert(0,Transition::GPTGet(ev.name.clone(), ev.goal.clone())); // *! passes the past goal
                        println!("[ Current extra transitions: {:?} ]", game_state.extra_transitions);
                    }
                };
            },
            Err(e) => {
                info!("[ Error: {:?} ]", e);

                let mut error_message = String::new();
                match e {
                    GPTError::RequestBuilderError => {
                        // Major issue, probably means the player corrupted something.
                        // This only occurs if the previous messages and goals can't be
                        // parsed, which is a big deal, because it's barely possible without
                        // some kinda cheat engine. Resave error.
                        error_message.push_str("Error parsing previous messages.\nFalling back to last save point...");
                    },
                    GPTError::LengthError => {
                        // The resulting body from OpenAI was too long. Don't know how
                        // this could ever even happen, but just in case, it'd be a resave
                        // error.
                        error_message.push_str("Response from OpenAI too long.\nContact the dev, he doesn't actually think this error is possible.\nFalling back to last save point...");
                    },
                    GPTError::IOError => {
                        // Likely means that the player lost or does not have internet
                        // connection. Alert the player to try again, and resave error.
                        error_message.push_str("Unable to send request to OpenAI after 5 attempts!\nPlease check your internet connection and firewall settings.\nFalling back to last save point...");
                    },
                    GPTError::OpenAIError => {
                        // This means the request faced a failure status code from OpenAI,
                        // meaning OpenAI is down or your api key is restricted / incorrect,
                        // meaning the engine should alert the player and resave error.
                        error_message.push_str("Received bad error code from OpenAI.\nVerify that your API key is correct and that you're not blacklisted or rate limited.\nFalling back to last save point...");
                    },
                    GPTError::UnparseableOpenAIResponse => {
                        // I literally have no idea how this could happen. If it does, that's
                        // probably indicitive of OpenAI changing the way their response JSON
                        // is formatted. Resave error.
                        error_message.push_str("Error parsing OpenAI response.\nContact the dev, he doesn't actually think this error is possible.\nFalling back to last save point...");
                    },
                    _ => panic!("Undefined behaviour")
                }

                let current_scene_id = game_state.current_scene_id.clone();
                info_text.0 = error_message.clone();

                game_state.extra_transitions.insert(0,Transition::Scene(current_scene_id));
            },
        }

        game_state.blocking = false;
    }

    /* GPT GET (Input) Event INITIALIZATION [Transition::GPTGet] */
    for ev in get_message.read() {
        game_state.blocking = true;

        // Make the parent typebox visible
        *typebox_visibility = Visibility::Visible;

        // Reset the typebox
        type_text.0 = String::from("");

        game_state.extra_transitions.insert(0,Transition::GPTSay(ev.past_character.clone(), ev.past_goal.clone())); // *! passes the past goal
    }

    /* GPT GET (Input) Event ONGOING [Transition::GPTGet] */
    // For each character input
    for event in input_messages.read() {
        match event.key_code {
            KeyCode::Backspace => { // If BACKSPACE, remove a character
                type_text.0.pop();
            },
            KeyCode::Enter => { // If ENTER, finish the prompt
                println!("[ Player finished typing: {} ]", name_text.0);

                // Hide textbox parent object
                *typebox_visibility = Visibility::Hidden;

                // Add the typed message
                let name = game_state.playername.clone();
                game_state.past_messages.push(CustomMessage {
                    role: String::from("user"),
                    content: format!("{}: {}", name, type_text.0.clone()),
                });

                // Allow transitions to be run again
                game_state.blocking = false;
            }
            _ => {},
        }
        if let Key::Character(char) = &event.logical_key {
            if type_text.0.len() < 310 {
                if let Some(c) = char.chars().next() {
                    type_text.0.push(c);
                }
            }
        }
    }

    /* STANDARD SAY EVENTS INITIALIZATION [Transition::Say] */
    for ev in event_message.read() {
        game_state.blocking = true;

        // Make the parent textbox visible
        *textbox_visibility = Visibility::Visible;

        // Update both the name and message text objects
        // Reset the scrolling timer
        scroll_stopwatch.0.set_elapsed(std::time::Duration::from_secs_f32(0.));

        // Update the name
        let name = if ev.name == "[_PLAYERNAME_]" { game_state.playername.clone() } else { ev.name.clone() };
        name_text.0 = name;

        // Update the message text and log it as a Message
        let role = if ev.name == "[_PLAYERNAME_]" { String::from("user") } else { String::from("assistant") };
        let mut name = format!("[{}]", game_state.playername.clone());
        let mut emotion = String::from("");
        for character in character_query.iter() {
            let emotion_str = match &character.emotion {
                Some(emotion) => emotion,
                None => continue
            };
            if character.name == ev.name {
                name = format!("[{}]", character.name);
                emotion = format!("[{}]", emotion_str);
            }
        }

        println!("MESSAGE {}", ev.message);
        game_state.past_messages.push(CustomMessage {
            role,
            content: format!("{}{}: {}", name, emotion, ev.message.clone()),
        });

        message_scroll_text_obj.message = ev.message.clone();

    }

    // (there needs to be a way to clean this up)
    // If the textbox is hidden, ignore the next section dedicated to updating it
    if *textbox_visibility == Visibility::Hidden {
        return;
    }

    // Take the original string from the message object
    let mut original_string: String = message_scroll_text_obj.message.clone();

    // Get the section of the string according to the ellapsed time
    let length: u32 = (scroll_stopwatch.0.elapsed_secs() * 50.) as u32;

    // Return the section and apply it to the text object
    original_string.truncate(length as usize);
    message_text.0 = original_string;

    let window = match window.get_single() {
        Ok(window) => window,
        Err(_) => {
            eprintln!("Error querying for window");
            return;
        }
    };
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
                // Skip message scrolling (bad code, should not be real)
                scroll_stopwatch.0.set_elapsed(std::time::Duration::from_secs_f32(100000000.));
                return;
            }
            println!("[ Player finished message ]");
            info_text.0 = String::from("");

            // Hide textbox parent object
            *textbox_visibility = Visibility::Hidden;

            // Allow transitions to be run again
            game_state.blocking = false;
        }
    }
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
) {
    for ev in change_messages.read() {
        for (gui_obj, mut current_sprite) in gui_query.iter_mut() {
            if gui_obj.id == ev.gui_id {
                current_sprite.image = game_state.gui_sprites.get(&ev.sprite_id)
                    .expect("GUI asset '{ev.sprite_id}' does not exist!")
                    .clone();
                println!("[ Set GUI asset '{}' to '{}']", ev.gui_id, ev.sprite_id);
            }
        }
    }
}
