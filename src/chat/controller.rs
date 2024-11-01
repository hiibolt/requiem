use crate::{intelligence::*, Character, Object, Transition, VisualNovelState};

use std::collections::HashMap;

use bevy::{prelude::*, sprite::Anchor, text::{BreakLineOn, Text2dBounds}, time::Stopwatch, window::PrimaryWindow};


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


pub struct ChatController;
impl Plugin for ChatController {
    fn build(&self, app: &mut App){
        app.insert_resource(ChatScrollStopwatch(Stopwatch::new()))
            .add_startup_system(import_gui_sprites)
            .add_startup_system(spawn_chatbox)
            .add_event::<GPTSayEvent>()
            .add_event::<GPTGetEvent>()
            .add_event::<CharacterSayEvent>()
            .add_event::<GUIChangeEvent>()
            .add_system(update_chatbox)
            .add_system(update_gui);
    }
}
fn import_gui_sprites( mut game_state: ResMut<VisualNovelState>, asset_server: Res<AssetServer> ){
    let mut gui_sprites = HashMap::<String, Handle<Image>>::new();
    let master_gui_dir = std::env::current_dir()
        .expect("Failed to get current directory!")
        .join("assets")
        .join("gui");
    let gui_sprite_paths = std::fs::read_dir(master_gui_dir)
        .expect("Unable to read outfit folders!")
        .filter_map(|entry| {
            if let Ok(entry) = entry {
                Some(entry.path())
            }else {
                info!("Unable to read file! Error: `{:?}`", entry);
                None
            }
        });
    for gui_sprite_path in gui_sprite_paths {
        let file_name = gui_sprite_path
            .file_stem().expect("Sprite file must have complete name!")
            .to_str().expect("Sprite file name must be valid UTF-8!")
            .to_string();
        println!("[ Importing GUI asset '{file_name}' ]");
        gui_sprites.insert(file_name, asset_server.load(gui_sprite_path));
    }
    game_state.gui_sprites = gui_sprites;
}
fn spawn_chatbox(mut commands: Commands, asset_server: Res<AssetServer>){
    // Spawn Backplate + Nameplate
    commands.spawn((
        Object {
            r#type: String::from("gui"),
            id: String::from("_textbox_background")
        },
        SpriteBundle {
            visibility: Visibility::Hidden,
            transform: Transform::from_xyz(0., -275., 2.),
            ..default()
        }
    ))
    .with_children(|parent| {
        parent.spawn((
            Object {
                r#type: String::from("gui"),
                id: String::from("_namebox_background")
            },
            SpriteBundle {
                visibility: Visibility::Inherited,
                transform: Transform::from_xyz(-270., 105., 2.)
                    .with_scale( Vec3 { x: 0.75, y: 0.75, z: 2. } ),
                ..default()
            }
        ));
        parent.spawn((
            Object {
                r#type: String::from("gui"),
                id: String::from("_name_text")
            },
            GUIScrollText {
                message: String::from("UNFILLED")
            },
            Text2dBundle {
                text: Text {
                    sections: vec![TextSection::new(
                        "UNFILLED",
                        TextStyle {
                            font: asset_server.load("fonts/ALLER.ttf"),
                            font_size: 40.0,
                            color: Color::WHITE,
                        })],
                    alignment: TextAlignment::Left,
                    linebreak_behaviour: BreakLineOn::WordBoundary
                },
                text_anchor: Anchor::TopLeft,
                transform: Transform::from_xyz(-305., 126., 3.),
                visibility: Visibility::Inherited,
                ..default()
            }
        ));
        parent.spawn((
            Object {
                r#type: String::from("gui"),
                id: String::from("_message_text")
            },
            GUIScrollText {
                message: String::from("UNFILLED")
            },
            Text2dBundle {
                text: Text {
                    sections: vec![TextSection::new(
                        "UNFILLED CHAT MESSAGE",
                        TextStyle {
                            font: asset_server.load("fonts/BOLDITALIC.ttf"),
                            font_size: 27.0,
                            color: Color::WHITE,
                        })],
                    alignment: TextAlignment::Left,
                    linebreak_behaviour: BreakLineOn::WordBoundary
                },
                text_anchor: Anchor::TopLeft,
                text_2d_bounds: Text2dBounds{ size: Vec2 { x: 700., y: 20000.} },
                transform: Transform::from_xyz(-350., 62., 3.),
                visibility: Visibility::Inherited,
                ..default()
            }
        ));
    });

    // Spawn typebox
    commands.spawn((
        Object {
            r#type: String::from("gui"),
            id: String::from("_typebox_background")
        },
        SpriteBundle {
            visibility: Visibility::Hidden,
            transform: Transform::from_xyz(0., -275., 2.),
            ..default()
        },
        TypeBox
    ))
    .with_children(|parent| {
        parent.spawn((
            Object {
                r#type: String::from("gui"),
                id: String::from("_type_text")
            },
            GUIScrollText {
                message: String::from("UNFILLED")
            },
            Text2dBundle {
                text: Text {
                    sections: vec![TextSection::new(
                        "Start typing...",
                        TextStyle {
                            font: asset_server.load("fonts/BOLDITALIC.ttf"),
                            font_size: 27.0,
                            color: Color::WHITE,
                        })],
                    alignment: TextAlignment::Left,
                    linebreak_behaviour: BreakLineOn::WordBoundary
                },
                text_anchor: Anchor::TopLeft,
                text_2d_bounds: Text2dBounds{ size: Vec2 { x: 700., y: 20000.} },
                transform: Transform::from_xyz(-350., 62., 3.),
                visibility: Visibility::Inherited,
                ..default()
            }
        ));
    });

    // Spawn info text
    commands.spawn((
        Object {
            r#type: String::from("gui"),
            id: String::from("_info_text")
        },
        GUIScrollText {
            message: String::from("WILL ALWAYS BE BLANK, YOU SHOULD CREATE DIFF TYPE")
        },
        Text2dBundle {
            text: Text {
                sections: vec![TextSection::new(
                    "",
                    TextStyle {
                        font: asset_server.load("fonts/BOLD.ttf"),
                        font_size: 50.,
                        color: Color::RED,
                    })],
                alignment: TextAlignment::Center,
                linebreak_behaviour: BreakLineOn::WordBoundary
            },
            text_anchor: Anchor::TopCenter,
            text_2d_bounds: Text2dBounds{ size: Vec2 { x: 700., y: 20000.} },
            transform: Transform::from_xyz(0., 302., 3.),
            visibility: Visibility::Visible,
            ..default()
        }
    ));
}
fn update_chatbox(
    mut event_message: EventReader<CharacterSayEvent>,
    mut gpt_message: EventReader<GPTSayEvent>,
    mut get_message: EventReader<GPTGetEvent>,
    character_query: Query<&Character>,
    mut visibility_query: Query<(&mut Visibility, &Object)>,
    mut text_object_query: Query<(&mut Text, &mut GUIScrollText, &Object), Without<TypeBox>>,
    mut scroll_stopwatch: ResMut<ChatScrollStopwatch>,

    mut events: EventReader<ReceivedCharacter>,

    mut game_state: ResMut<VisualNovelState>,

    time: Res<Time>,
    window: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<Input<MouseButton>>,
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
    let mut name_text_option: Option<&mut Text> = None;
    let mut type_text_option: Option<&mut Text> = None;
    let mut info_text_option: Option<&mut Text> = None;
    let mut message_text_option: Option<&mut Text> = None;
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
    let to_tick = if time.delta_seconds() > 1. { std::time::Duration::from_secs_f32(0.) } else { time.delta() };
    scroll_stopwatch.0.tick(to_tick);

    /* GPT EVENTS [Transition::GPTSay] */
    for ev in gpt_message.iter() {
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
                info_text.sections[0].value = error_message.clone();

                game_state.extra_transitions.insert(0,Transition::Scene(current_scene_id));
            },
        }

        game_state.blocking = false;
    }

    /* GPT GET (Input) Event INITIALIZATION [Transition::GPTGet] */
    for ev in get_message.iter() {
        game_state.blocking = true;

        // Make the parent typebox visible
        *typebox_visibility = Visibility::Visible;

        // Reset the typebox
        type_text.sections[0].value = String::from("");

        game_state.extra_transitions.insert(0,Transition::GPTSay(ev.past_character.clone(), ev.past_goal.clone())); // *! passes the past goal
    }

    /* GPT GET (Input) Event ONGOING [Transition::GPTGet] */
    // For each character input
    for event in events.iter() {
        match event.char.escape_default().collect::<String>().as_str() {
            "\\u{8}" => { // If BACKSPACE, remove a character
                type_text.sections[0].value.pop();
            },
            "\\r" => { // If ENTER, finish the prompt
                println!("[ Player finished typing: {} ]", name_text.sections[0].value);

                // Hide textbox parent object
                *typebox_visibility = Visibility::Hidden;

                // Add the typed message
                let name = game_state.playername.clone();
                game_state.past_messages.push( Message {
                    role: String::from("user"),
                    content: format!("{}: {}", name, type_text.sections[0].value.clone()),
                });

                // Allow transitions to be run again
                game_state.blocking = false;
            }
            _ => if type_text.sections[0].value.len() < 310 {
                type_text.sections[0].value.push(event.char)
            },
        }
    }

    /* STANDARD SAY EVENTS INITIALIZATION [Transition::Say] */
    for ev in event_message.iter() {
        game_state.blocking = true;

        // Make the parent textbox visible
        *textbox_visibility = Visibility::Visible;

        // Update both the name and message text objects
        // Reset the scrolling timer
        scroll_stopwatch.0.set_elapsed(std::time::Duration::from_secs_f32(0.));

        // Update the name
        let name = if ev.name == "[_PLAYERNAME_]" { game_state.playername.clone() } else { ev.name.clone() };
        name_text.sections[0].value = name;

        // Update the message text and log it as a Message
        let role = if ev.name == "[_PLAYERNAME_]" { String::from("user") } else { String::from("assistant") };
        let mut name = format!("[{}]", game_state.playername.clone());
        let mut emotion = String::from("");
        for character in character_query.iter() {
            if character.name == ev.name {
                name = format!("[{}]", character.name);
                emotion = format!("[{}]", character.emotion);
            }
        }

        game_state.past_messages.push( Message {
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
    message_text.sections[0].value = original_string;

    if let Some(position) = window.single().cursor_position() {
        let resolution = &window.single().resolution;
        let textbox_bounds: [f32; 4] = [
            ( resolution.width() / 2. ) - ( 796. / 2. ),
            ( resolution.width() / 2. ) + ( 796. / 2. ),
            ( resolution.height() / 2. ) - ( 155. / 2. ) - ( 275. ),
            ( resolution.height() / 2. ) + ( 155. / 2. ) - ( 275. ),
        ];
        if ( position.x > textbox_bounds[0] && position.x < textbox_bounds[1] ) && ( position.y > textbox_bounds[2] && position.y < textbox_bounds[3] ) && buttons.just_pressed(MouseButton::Left) {
            if length < message_scroll_text_obj.message.len() as u32 {
                // Skip message scrolling (bad code, should not be real)
                scroll_stopwatch.0.set_elapsed(std::time::Duration::from_secs_f32(100000000.));
                return;
            }
            println!("[ Player finished message ]");
            info_text.sections[0].value = String::from("");

            // Hide textbox parent object
            *textbox_visibility = Visibility::Hidden;

            // Allow transitions to be run again
            game_state.blocking = false;
        }
    }
            
        

}
fn update_gui(
    mut event_change: EventReader<GUIChangeEvent>,
    mut gui_query: Query<(&Object, &mut Handle<Image>)>,

    game_state: Res<VisualNovelState>
) {
    for ev in event_change.iter() {
        for (gui_obj, mut current_sprite) in gui_query.iter_mut() {
            if gui_obj.id == ev.gui_id {
                *current_sprite = game_state.gui_sprites.get(&ev.sprite_id)
                    .expect("GUI asset '{ev.sprite_id}' does not exist!")
                    .clone();
                println!("[ Set GUI asset '{}' to '{}']", ev.gui_id, ev.sprite_id);
            }
        }
    }
}