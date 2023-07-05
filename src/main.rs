use bevy::{
    prelude::*,
    window::*,
    asset::{ Handle },
    sprite::{ Anchor },
    time::{ Stopwatch },
    text::{ BreakLineOn, Text2dBounds }
};
use std::fs;
use std::vec::IntoIter;
use std::collections::HashMap;
use std::env;

use regex::Regex;

use json::parse;
use serde::{ Serialize, Deserialize };



#[derive(Resource, Default)]
struct VisualNovelState {
    // Player-designated constants
    playername: String,
    api_key: String,

    gui_sprites: HashMap<String, Handle<Image>>,

    transitions_iter: IntoIter<Transition>,
    
    extra_transitions: Vec<Transition>,

    past_messages: Vec<Message>,

    blocking: bool,
}


/*
 _                _                                   _
| |__   __ _  ___| | ____ _ _ __ ___  _   _ _ __   __| |
| '_ \ / _` |/ __| |/ / _` | '__/ _ \| | | | '_ \ / _` |
| |_) | (_| | (__|   | (_| | | | (_) | |_| | | | | (_| |
|_.__/ \__,_|\___|_|\_\__, |_|  \___/ \__,_|_| |_|\__,_|
                      |___/
*/
/* Components */
#[derive(Component)]
struct Background {
    background_sprites: HashMap::<String, Handle<Image>>
}

/* Events */
struct BackgroundChangeEvent {
    background_id: String
}

pub struct BackgroundController;
impl Plugin for BackgroundController {
    fn build(&self, app: &mut App){
        app.add_startup_system(import_backgrounds)
            .add_event::<BackgroundChangeEvent>()
            .add_system(update_background);
    }
}
fn import_backgrounds(mut commands: Commands, asset_server: Res<AssetServer>){
    let mut background_sprites: HashMap<String, Handle<Image>>= HashMap::new();

    let master_backgrounds_dir = std::env::current_dir()
        .expect("Failed to get current directory!")
        .join("assets")
        .join("backgrounds");
    let background_paths = fs::read_dir(master_backgrounds_dir)
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
        Background {
            background_sprites,
        }, 
        SpriteBundle {
            transform: Transform::IDENTITY,
            ..default()
        }
    ));
}
fn update_background(
    mut background_query: Query<(
        &Background, 
        &mut Handle<Image>
    ), (With<Background>, Without<Character>)>,

    mut background_change_event: EventReader<BackgroundChangeEvent>,
){
    for ev in background_change_event.iter() {
        for (background_obj, mut current_sprite) in background_query.iter_mut() {
            *current_sprite = background_obj.background_sprites.get(&ev.background_id)
                .expect("'{character.outfit}' attribute does not exist!")
                .clone();
            println!("[ Set background to '{}']", ev.background_id);
        }
    }
}

/*
      _           _
  ___| |__   __ _| |_
 / __| '_ \ / _` | __|
| (__| | | | (_| | |_
 \___|_| |_|\__,_|\__|
*/
/* Components */
#[derive(Component)]
struct GUISprite {
    id: String,
}
#[derive(Component)]
struct GUIScrollText {
    id: String,
    message: String
}
#[derive(Component)]
struct TypeBox;

/* Resources */
#[derive(Resource)]
struct ChatScrollStopwatch(Stopwatch);

/* Events */
struct GPTGetEvent {
    past_character: String,
    past_goal: String
}
struct GPTSayEvent {
    name: String,
    goal: String,
    advice: Option<String>
}
struct CharacterSayEvent {
    name: String,
    message: String
}
struct GUIChangeEvent {
    gui_id: String,
    sprite_id: String
}

/* Custom Types */
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Message {
    role: String,
    content: String
}
#[derive(Deserialize, Debug)]
struct ChatChoice {
    //index: usize,
    message: Message,
    //finish_reason: String
}
#[derive(Deserialize, Debug)]
struct CompletionChoice {
    //index: usize,
    text: String,
    //finish_reason: String
}
#[derive(Deserialize, Debug)]
struct Usage {
    //prompt_tokens: usize,
    //completion_tokens: usize,
    total_tokens: usize
}
#[derive(Deserialize, Debug)]
struct ChatResponse {
    //id: Option<String>,
    //object: Option<String>,
    //created: Option<u64>,
    //model: Option<String>,
    choices: Vec<ChatChoice>,
    usage: Option<Usage>
}
#[derive(Deserialize, Debug)]
struct CompletionResponse {
    //id: Option<String>,
    //object: Option<String>,
    //created: Option<u64>,
    //model: Option<String>,
    choices: Vec<CompletionChoice>,
    usage: Option<Usage>
}
#[derive(Serialize, Debug)]
struct GPTTurboRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32
}
#[derive(Serialize, Debug)]
struct CompletionRequest {
    model: String,
    prompt: String,
    temperature: f32,
    max_tokens: usize
}
#[derive(Deserialize, Debug)]
struct GoalResponse {
    reason: Option<String>,
    goal_status: String,
    advice: Option<String>,
}

#[derive(Debug)]
enum GPTError {
    RequestBuilderError,
    LengthError,
    IOError,
    OpenAIError,
    UnparseableOpenAIResponse,
    Null
}
fn message_context_to_stringified_request(character: &Character, game_state: &VisualNovelState, event: &GPTSayEvent) -> Result<String, GPTError>{
    // Build the prompt for the request
    let mut messages = Vec::<Message>::new();
    messages.push(Message { 
        role: String::from("system"),
        content: character.description.clone(),
    });
    messages.push(Message { 
        role: String::from("system"),
        content: format!("{}'s goal: `{}`. Your goal is NOT yet achieved.", character.name, event.goal.clone())
    });
    messages.push(Message { 
        role: String::from("system"),
        content: format!("Generate two messages. Format: `[{}][{}]: blah blah blah etc`", character.name, character.emotions.join(" | "))
    });
    messages.extend_from_slice(game_state.past_messages.as_slice());

    // Build the request object to be serialized
    let request = GPTTurboRequest {
        model: String::from("gpt-3.5-turbo"),
        messages,
        temperature: 1.,
    };

    // Serialize the request
    return serde_json::to_string(&request)
        .map_err(|_| GPTError::RequestBuilderError);

}
fn query_gpt_turbo(request_string: &String, api_key: &String) -> Result<String, GPTError> {
    let mut result: Result<String, GPTError> = Err(GPTError::Null);
    for attempt in 1..=5 {
        info!("Attempt {} of 5", attempt);
        match ureq::post("https://api.openai.com/v1/chat/completions")
            .set("Authorization", &format!("Bearer {}", api_key))
            .set("Content-Type", "application/json")
            .send_string(request_string)
        {
            Ok(successful_post) => {
                if let Ok(string) = successful_post.into_string() {
                    result = Ok(string);
                    break;
                }else if attempt == 5 {
                    result = Err(GPTError::LengthError);
                    break;
                }
            },
            Err(e) => {
                if attempt == 5 {
                    match e {
                        ureq::Error::Status(_, _) => {
                            result = Err(GPTError::OpenAIError);
                        },
                        ureq::Error::Transport(_) => {
                            result = Err(GPTError::IOError);
                        }
                    }
                    break;
                }
            }
        }
    }
    result
}
fn generate_chat_transitions(character: &Character, game_state: &VisualNovelState, event: &GPTSayEvent) -> Vec<Transition> {
    let mut ret = Vec::<Transition>::new();

    // Serialize the request
    let serialized_request: String = message_context_to_stringified_request(character, game_state, event).expect("TODO!");

    // Make the request
    println!("[ Sending GPT request to OpenAI ]");
    let response_object: ChatResponse = query_gpt_turbo(&serialized_request, &game_state.api_key)
        .and_then(|response| {
            serde_json::from_str(&response).or(Err(GPTError::UnparseableOpenAIResponse))
        })
        .unwrap();

    // Parse the response
    let response_message = response_object.choices[0].message.content.clone();

    println!("[ Response: {} ]", response_message);
    if let Some(usage) = response_object.usage {
        println!("[ Usage: {} ]", usage.total_tokens.clone());
    }
    // Matches [...][...]: ...
    let message_structure = Regex::new(r"\[(.+)\]\[(.+)\]: ([\S\s]+)").expect("Please re-write the message structure regex!");

    /* 
    Split the response by each message
        ([...][...]: ...)

        This allows for if the model generates dialog for
        multiple characters, or on the behalf of the user
    */
    let all_messages_groups_string = message_structure.replace_all(&response_message, |caps: &regex::Captures| {
            format!("~<>>[{}][{}]: {}", &caps[1], &caps[2], &caps[3])
        });
    let all_messages_groups = all_messages_groups_string
        .split("~<>>")
        .filter(|line| !line.is_empty())
        .collect::<Vec<&str>>();

    for message_group in all_messages_groups {
        println!("[ MESSAGE GROUP HEADER ]");
        /* Splits the INDIVIDUAL message group by "\n" 
            Example:
            [...][...]: blah blah blah
            blah blah blah
            blah blah blah
            vvvv   translates to  vvvv
            [...][...]: blah blah blah
            [...][...]: blah blah blah
            [...][...]: blah blah blah
        */
        let mut extract_message = || -> Result<&str, String> {
            let message_captures = message_structure
                .captures(message_group)
                .ok_or("Couldn't find message!")?;
            let character_name: String = message_captures.get(1)
                .ok_or("Couldn't find character name!")?
                .as_str()
                .to_owned();
            let emotion: String = message_captures.get(2)
                .ok_or("Couldn't find emotion!")?
                .as_str()
                .to_uppercase();
            let response_unsplit = message_captures.get(3)
                .ok_or("Couldn't find response!")?
                .as_str();

            ret.push(Transition::SetEmotion(character_name,emotion));

            Ok(response_unsplit)
        };
        
        let responses_split: Vec<&str> = extract_message()
            .unwrap_or(message_group)
            .split("\n")
            .filter(|line| !line.is_empty())
            .collect();
        
        // Update the emotion
        for message in responses_split {
            println!("[ NEW MESSAGE: {} ]", message);
            ret.push(Transition::Say(String::from(event.name.clone()),String::from(message)));
        }
    }
    ret
}
fn determine_goal_status(character: &Character, game_state: &VisualNovelState, event: &GPTSayEvent) -> Option<bool> {
    // Build the prompt for the request
    let prompt = format!("Decide whether {} achieved their goal of \"{}\". Give advice to the character on what to do, and reason for why the goal isn't completed in JSON form.
    
    Conversation:
    {}

    Response example: 
    {{\n\t\"character\": \"{}\",\n\t\"reason\": \"reason why goal is or isnt completed\",\"advice\":\"advice for completing goal\" | null,\n\t\"goal_status\": \"NO\"\n}}
    Possible goal statuses: \"YES\", \"NO\"
    
    Final Reponse:
    {{\n\t\"character\": \"{}\",
    ", character.name, event.goal, game_state.past_messages.clone().iter().map(|item| item.content.clone()).collect::<String>(), character.name, character.name );
    // Build the request object to be serialized
    let request = CompletionRequest {
        model: String::from("text-davinci-003"),
        prompt,
        temperature: 1.,
        max_tokens: 300,
    };

    // Serialize the request
    let serialized_request = serde_json::to_string(&request).ok()?;

    println!("[ Sending GPT GOAL CHECK request to OpenAI ]");

    // Make the request
    let resp: String = ureq::post("https://api.openai.com/v1/completions")
        .set("Authorization", &format!("Bearer {}", game_state.api_key))
        .set("Content-Type", "application/json")
        .send_string(&serialized_request)
        .ok()?
        .into_string()
        .ok()?;

    // Parse the response
    let response_object: CompletionResponse = serde_json::from_str(&resp).ok()?;
    let response_message = response_object.choices[0].text.clone();

    if let Some(usage) = response_object.usage {
        println!("[ Usage: {} ]", usage.total_tokens.clone());
    }
    println!("[ Response: {} ]", response_message);
    // Extract the goal status from the response
    let goal_response_object: GoalResponse = serde_json::from_str( 
        &(String::from("{") + &response_message.replace(|c: char| if c == '\n' { true } else { c.is_whitespace() }, "")) 
    )
        .ok()?;
    Some(goal_response_object.goal_status == "YES")
}
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
    let gui_sprite_paths = fs::read_dir(master_gui_dir)
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
        GUISprite {
            id: String::from("textbox_background")
        },
        SpriteBundle {
            visibility: Visibility::Hidden,
            transform: Transform::from_xyz(0., -275., 2.),
            ..default()
        }
    ))
    .with_children(|parent| {
        parent.spawn((
            GUISprite {
                id: String::from("namebox_background")
            },
            SpriteBundle {
                visibility: Visibility::Inherited,
                transform: Transform::from_xyz(-270., 105., 2.)
                    .with_scale( Vec3 { x: 0.75, y: 0.75, z: 2. } ),
                ..default()
            }
        ));
        parent.spawn((
            GUIScrollText {
                id: String::from("name_text"),
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
            GUIScrollText {
                id: String::from("message_text"),
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
        GUISprite {
            id: String::from("typebox_background")
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
            GUIScrollText {
                id: String::from("type_text"),
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
}
fn update_chatbox(
    mut event_message: EventReader<CharacterSayEvent>,
    mut gpt_message: EventReader<GPTSayEvent>,
    mut get_message: EventReader<GPTGetEvent>,
    character_query: Query<&Character>,
    mut text_visibility_query: Query<(&mut Visibility, &GUISprite), Without<TypeBox>>,
    mut typing_visibility_query: Query<(&mut Visibility, &GUISprite), With<TypeBox>>,
    mut text_object_query: Query<(&mut Text, &mut GUIScrollText), Without<TypeBox>>,
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
    // Reference to SPECIFICALLY the typing text display object
    let mut name_text_option: Option<&mut Text> = None;
    for (name_text_, scroll_text_obj) in text_object_query.iter_mut() {
        if scroll_text_obj.id == "type_text" {
            name_text_option = Some(name_text_.into_inner());
        }
    }
    let name_text = name_text_option.expect("MISSING GUI OBJECT WITH ID 'type_text'!");

    // Reference to SPECIFICALLY the typing text display object
    let mut typebox_visibility_option: Option<&mut Visibility> = None;
    for (visibility, text_box_object) in typing_visibility_query.iter_mut() {
        if text_box_object.id == "typebox_background" {
            typebox_visibility_option = Some(visibility.into_inner());
        }
    }
    let typebox_visibility = typebox_visibility_option.expect("MISSING GUI OBJECT WITH ID 'typebox_background'!");

    
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
        let mut ret = generate_chat_transitions(&character, &game_state, &ev);
        game_state.extra_transitions.append(&mut ret);
        
        /* GPT CHECK - CHECK IF THE CHARACTER ACHIEVED THEIR GOAL! */
        // Any errors here should literally result in continuation of the game.
        // It's way easier to let the next prompt be generated, 
        // and let the player see the dialog that was just generated anyway
        // (plus, it could be OpenAI rate limits)
        if let Some(goal_status) = determine_goal_status(&character, &game_state, &ev){
            println!("[ Goal Status: {} ]", goal_status);
            if !goal_status {
                game_state.extra_transitions.insert(0,Transition::GPTGet(ev.name.clone(), ev.goal.clone())); // *! passes the past goal
            }
        };

        game_state.blocking = false;
    }

    /* GPT GET (Input) Event INITIALIZATION [Transition::GPTGet] */
    for ev in get_message.iter() {
        game_state.blocking = true;

        // Make the parent typebox visible
        *typebox_visibility = Visibility::Visible;

        // Reset the typebox
        name_text.sections[0].value = String::from("");

        game_state.extra_transitions.insert(0,Transition::GPTSay(ev.past_character.clone(), ev.past_goal.clone())); // *! passes the past goal
    }

    /* GPT GET (Input) Event ONGOING [Transition::GPTGet] */
    // For each character input
    for event in events.iter() {
        match event.char.escape_default().collect::<String>().as_str() {
            "\\u{8}" => { // If BACKSPACE, remove a character
                name_text.sections[0].value.pop();
            },
            "\\r" => { // If ENTER, finish the prompt
                println!("[ Player finished typing: {} ]", name_text.sections[0].value);

                // Hide textbox parent object
                *typebox_visibility = Visibility::Hidden;

                // Add the typed message
                let name = game_state.playername.clone();
                game_state.past_messages.push( Message {
                    role: String::from("user"),
                    content: format!("{}: {}", name, name_text.sections[0].value.clone()),
                });

                // Allow transitions to be run again
                game_state.blocking = false;
            }
            _ => if name_text.sections[0].value.len() < 310 {
                name_text.sections[0].value.push(event.char)
            },
        }
    }

    /* STANDARD SAY EVENTS INITIALIZATION [Transition::Say] */
    for ev in event_message.iter() {
        game_state.blocking = true;

        // Make the parent textbox visible
        for (mut visibility, text_box_object) in text_visibility_query.iter_mut() {
            if text_box_object.id == "textbox_background" {
                *visibility = Visibility::Visible;
            }
        }

        // Update both the name and message text objects
        for (mut name_text, mut scroll_text_obj) in text_object_query.iter_mut() {
            // Reset the scrolling timer
            scroll_stopwatch.0.set_elapsed(std::time::Duration::from_secs_f32(0.));

            // Update the name
            if scroll_text_obj.id == "name_text" {
                let name = if ev.name == "[_PLAYERNAME_]" { game_state.playername.clone() } else { ev.name.clone() };
                name_text.sections[0].value = name;
            }

            // Update the message text and log it as a Message
            if scroll_text_obj.id == "message_text" {
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

                scroll_text_obj.message = ev.message.clone();
            }
        }
    }

    // (there needs to be a way to clean this up)
    // If the textbox is hidden, ignore the next section dedicated to updating it
    for (visibility, text_box_object) in text_visibility_query.iter() {
        if text_box_object.id == "textbox_background" && visibility == Visibility::Hidden {
            return;
        }
    }
    // you need to find a way to remove the number of indent levels bro
    for (mut name_text, scroll_text_obj) in text_object_query.iter_mut() {
        if scroll_text_obj.id == "message_text" {
            // Take the original string from the message object
            let mut original_string: String = scroll_text_obj.message.clone();

            // Get the section of the string according to the ellapsed time
            let length: u32 = (scroll_stopwatch.0.elapsed_secs() * 50.) as u32;

            // Return the section and apply it to the text object
            original_string.truncate(length as usize);
            name_text.sections[0].value = original_string;

            if let Some(position) = window.single().cursor_position() {
                let resolution = &window.single().resolution;
                let textbox_bounds: [f32; 4] = [
                    ( resolution.width() / 2. ) - ( 796. / 2. ),
                    ( resolution.width() / 2. ) + ( 796. / 2. ),
                    ( resolution.height() / 2. ) - ( 155. / 2. ) - ( 275. ),
                    ( resolution.height() / 2. ) + ( 155. / 2. ) - ( 275. ),
                ];
                if ( position.x > textbox_bounds[0] && position.x < textbox_bounds[1] ) && ( position.y > textbox_bounds[2] && position.y < textbox_bounds[3] ) && buttons.just_pressed(MouseButton::Left) {
                    if length < scroll_text_obj.message.len() as u32 {
                        // Skip message scrolling (bad code, should not be real)
                        scroll_stopwatch.0.set_elapsed(std::time::Duration::from_secs_f32(100000000.));
                        return;
                    }
                    println!("[ Player finished message ]");

                    // Hide textbox parent object
                    for (mut visibility, text_box_object) in text_visibility_query.iter_mut() {
                        if text_box_object.id == "textbox_background" {
                            *visibility = Visibility::Hidden;
                        }
                    }

                    // Allow transitions to be run again
                    game_state.blocking = false;
                }
            }
            
        }
    }

}
fn update_gui(
    mut event_change: EventReader<GUIChangeEvent>,
    mut gui_query: Query<(&GUISprite, &mut Handle<Image>)>,

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


/*
      _                          _
  ___| |__   __ _ _ __ __ _  ___| |_ ___ _ __
 / __| '_ \ / _` | '__/ _` |/ __| __/ _ | '__|
| (__| | | | (_| | | | (_| | (__| ||  __| |
 \___|_| |_|\__,_|_|  \__,_|\___|\__\___|_|
*/
/* Components */
#[derive(Component)]
struct Character {
    name: String,
    outfit: String,
    emotion: String,
    description: String,
    emotions: Vec<String>
}
#[derive(Component)]
struct CharacterSprites {
    outfits: HashMap::<String, HashMap<String, Handle<Image>>>,
}

/* Resources */
#[derive(Resource)]
struct OpacityFadeTimer(Timer);

/* Events */
struct EmotionChangeEvent {
    name: String,
    emotion: String
}

pub struct CharacterController;
impl Plugin for CharacterController {
    fn build(&self, app: &mut App){
        app.insert_resource(OpacityFadeTimer(Timer::from_seconds(0.005, TimerMode::Repeating)))
            .add_startup_system(import_characters)
            .add_event::<EmotionChangeEvent>()
            .add_system(update_characters);
    }
}
fn import_characters(mut commands: Commands, asset_server: Res<AssetServer>){
    /* Character Setup */
    // Asset Gathering
    let mut outfits = HashMap::<String, HashMap<String, Handle<Image>>>::new();
    let master_character_dir = std::env::current_dir()
        .expect("Failed to get current directory!")
        .join("assets")
        .join("characters")
        .join("Nayu");
    let outfit_dirs = fs::read_dir(master_character_dir)
        .expect("Unable to read outfit folders!")
        .filter_map(|entry_result| {
            let entry = entry_result.ok()?;
            if let is_dir = entry.file_type().ok()?.is_dir() {
                match is_dir {
                    true => return Some(entry.path()),
                    false => return None
                }
            } else {
                None
            }
        });
    for outfit_dir in outfit_dirs {
        let mut emotion_sprites = HashMap::<String, Handle<Image>>::new();
        let outfit_name = outfit_dir
            .file_name().expect("No directory name!")
            .to_str().expect("Malformed UTF-8 in directory name, please verify it meets UTF-8 validity!")
            .to_owned();
        
        let sprite_paths = fs::read_dir(outfit_dir)
            .expect("No character data!")
            .filter_map(|entry| {
                if let Ok(entry) = entry {
                    Some(entry.path())
                }else{
                    info!("Failed to read file data of `{:?}`!", entry);
                    None
                }
            });
        for sprite_path in sprite_paths {
            let sprite_name = sprite_path
                .file_stem().expect("No file name! Your emotion sprites MUST have a label to be able to be referred to!")
                .to_str().expect("Malformed UTF-8 in file name, please verify it meets UTF-8 validity!")
                .to_string();
            let file_texture = asset_server.load(sprite_path);

            println!("Imported sprite '{}' for outfit '{}'", sprite_name, outfit_name);
            emotion_sprites.insert(sprite_name, file_texture);
        }
        outfits.insert(outfit_name, emotion_sprites);
    }
    // Character Info Gathering
    let character_string: String = fs::read_to_string(std::env::current_dir()
            .expect("Failed to get current directory!")
            .join("assets")
            .join("characters")
            .join("Nayu")
            .join("character.json"))
        .expect("Issue reading file!");
    let parsed_character = parse(&character_string).expect("Malformed JSON!");

    let name = parsed_character["name"].as_str().expect("Missing 'name' attribute").to_owned();
    let outfit = parsed_character["default_outfit"].as_str().expect("Missing 'name' attribute").to_owned();
    let emotion = parsed_character["default_emotion"].as_str().expect("Missing 'name' attribute").to_owned();

    commands.spawn((
        Character {
            name,
            outfit: outfit.clone(),
            emotion: outfit.clone(),
            description: parsed_character["description"].as_str().expect("Missing 'name' attribute").to_owned(),
            emotions: parsed_character["emotions"]
                .members()
                .map(|entry| entry.as_str()
                    .expect("Missing 'name' attribute")
                    .to_owned()
                ).collect::<Vec<String>>(),
        },
        SpriteBundle {
            texture: outfits.get(&outfit)
                .expect("'{character.outfit}' attribute does not exist!")
                .get(&emotion)
                .expect("'default_emotion' atttribute does not exist!")
                .clone(),
            transform: Transform::IDENTITY
                .with_translation(Vec3 { x:0., y:-40., z:1. } )
                .with_scale(Vec3 { x:0.75, y:0.75, z:1. } ),
            ..default()
        },
        CharacterSprites { outfits }
    ));
}
fn update_characters(
    mut character_query: Query<(
        &mut Character, 
        &CharacterSprites,
        &mut Handle<Image>
    ), (With<Character>, Without<Background>)>,
    mut event_emotion_change: EventReader<EmotionChangeEvent>,
    
    _text_object_query: Query<(&mut Text, &mut GUIScrollText)>,
    _scroll_stopwatch: ResMut<ChatScrollStopwatch>,
){
    for ev in event_emotion_change.iter() {
        for (mut character, sprites, mut current_sprite) in character_query.iter_mut() {
            if character.name == ev.name {
                character.emotion = ev.emotion.to_owned();
                *current_sprite = sprites.outfits.get(&character.outfit)
                    .expect("'{character.outfit}' attribute does not exist!")
                    .get(&character.emotion)
                    .expect("'default_emotion' atttribute does not exist!")
                    .clone();
                println!("[ Set emotion of '{}' to '{}']", ev.name, ev.emotion);
            }
        }
    }
}






/*
                           _ _
  ___ ___  _ __ ___  _ __ (_| | ___ _ __
 / __/ _ \| '_ ` _ \| '_ \| | |/ _ | '__|
| (_| (_) | | | | | | |_) | | |  __| |
 \___\___/|_| |_| |_| .__/|_|_|\___|_|
                    |_|
*/
/* Custom Types */
enum Transition {
    Say(String, String),
    SetEmotion(String, String),
    SetBackground(String),
    SetGUI(String, String),
    GPTGet(String, String),
    GPTSay(String, String),
    Log(String),
    End
}
impl Transition {
    fn call(
        &self, 
        character_say_event: &mut EventWriter<CharacterSayEvent>,
        emotion_change_event: &mut EventWriter<EmotionChangeEvent>,
        background_change_event: &mut EventWriter<BackgroundChangeEvent>,
        gui_change_event: &mut EventWriter<GUIChangeEvent>,
        gpt_say_event: &mut EventWriter<GPTSayEvent>,
        gpt_get_event: &mut EventWriter<GPTGetEvent>,

        game_state: &mut ResMut<VisualNovelState>, 
    ) {
        match self {
            Transition::Say(character_name, msg) => {
                info!("Calling Transition::Say");
                character_say_event.send(CharacterSayEvent {
                    name: character_name.to_owned(),
                    message: msg.to_owned()
                });
                game_state.blocking = true;
            },
            Transition::SetEmotion(character_name, emotion) => {
                info!("Calling Transition::SetEmotion");
                emotion_change_event.send(EmotionChangeEvent {
                    name: character_name.to_owned(),
                    emotion: emotion.to_owned()
                });
            }
            Transition::SetBackground(background_id) => {
                info!("Calling Transition::SetBackground");
                background_change_event.send(BackgroundChangeEvent {
                    background_id: background_id.to_owned()
                });
            },
            Transition::SetGUI(gui_id, sprite_id) => {
                info!("Calling Transition::SetGUI");
                gui_change_event.send(GUIChangeEvent {
                    gui_id: gui_id.to_owned(),
                    sprite_id: sprite_id.to_owned()
                });
            },
            Transition::GPTSay(character_name, character_goal) => {
                info!("Calling Transition::GPTSay");
                game_state.blocking = true;
                gpt_say_event.send(GPTSayEvent {
                    name: character_name.to_owned(),
                    goal: character_goal.to_owned(),
                    advice: None
                });
            },
            Transition::GPTGet(past_character, past_goal) => {
                info!("Calling Transition::GPTGet");
                game_state.blocking = true;
                gpt_get_event.send(GPTGetEvent {past_character: past_character.clone(), past_goal: past_goal.clone()});
            },
            Transition::Log(msg) => println!("{msg}"),
            Transition::End => {
                todo!();
            }
        }
    }
}



pub struct Compiler;
impl Plugin for Compiler {
    fn build(&self, app: &mut App){
        app.add_startup_system(pre_compile)
            .add_system(run_transitions);
    }
}
fn pre_compile( mut game_state: ResMut<VisualNovelState>){
    info!("Starting pre-compilation");
    let command_structure: Regex = Regex::new(r"(?<cmd_id>\w+)[\s$]").expect("Bad command_structure Regex compilation! Contact the developer.");
    let argument_structure: Regex = Regex::new(r"(?<arg_id>\w+)=`(?<arg_content>[^`]*)`").expect("Bad argument_structure Regex compilation! Contact the developer.");

    // Compile Script into a vector Transitions, then create an iterator over them
    let full_script_string: String = fs::read_to_string(std::env::current_dir()
            .expect("Failed to get current directory!")
            .join("assets")
            .join("scripts")
            .join("script.txt"))
        .expect("Issue reading file!");
    let transitions: Vec<Transition> = full_script_string.lines().map(move |line| {
        println!("[ Compiling  `{line}` ]");

        let mut command_options: HashMap<String, String> = HashMap::new();

        // Remove the command identifier seperately
        let cmd_id = command_structure.captures(line)
            .expect("Line `{line}` is blank!")
            .name("cmd_id")
            .expect("Line `{line}` is missing a basic command identifier! Example: `log`")
            .as_str();
        println!("CMD: `{cmd_id}`");
        
        
        // Adds each option from the command to the options hashmap
        for capture in argument_structure.captures_iter(line) {
            let arg_id: &str = capture.name("arg_id")
                .expect("Line `{line}` is missing a basic command identifier! Example: `log`")
                .as_str();
            let arg_content: &str = capture.name("arg_content")
                .expect("Line `{line}` is missing a basic command identifier! Example: `log`")
                .as_str();

            println!("Field - `{}` with content `{}`", arg_id, arg_content);
            
            command_options.insert(arg_id.to_owned(), arg_content.to_owned());
        }

        // Try to run the command
        match cmd_id {
            "log" => {
                let msg = command_options.get("msg")
                    .expect("Missing 'msg' option!")
                    .to_owned();
                Transition::Log(msg)
            },
            "say" => {
                let character_id = command_options.get("character")
                    .expect("Missing 'character' option!")
                    .to_owned();
                let msg = command_options.get("msg")
                    .expect("Missing 'msg' option!")
                    .to_owned();
                Transition::Say(character_id, msg)
            },
            "psay" => {
                let msg = command_options.get("msg")
                    .expect("Missing 'msg' option!")
                    .to_owned();
                Transition::Say(String::from("[_PLAYERNAME_]"), msg)
            },
            "gpt" => {
                let character_name = command_options.get("character")
                    .expect("Missing 'character' option!")
                    .to_owned();
                let character_goal = command_options.get("goal")
                    .expect("Missing 'goal' option!")
                    .to_owned();
                Transition::GPTSay(character_name, character_goal)
            },
            "set" => {
                let type_of = command_options.get("type")
                    .expect("Missing 'type' option!")
                    .as_str();
                match type_of {
                    "emotion" => {
                        let character_name = command_options.get("character")
                            .expect("Missing 'character' option required for type 'emotion'!")
                            .to_owned();
                        let emotion = command_options.get("emotion")
                            .expect("Missing 'emotion' option required for type 'emotion'!")
                            .to_owned();
                        Transition::SetEmotion(character_name, emotion)
                    },
                    "background" => {
                        let background_id = command_options.get("background")
                            .expect("Missing 'background' option required for type 'background'!")
                            .to_owned();
                        Transition::SetBackground( background_id )
                    }
                    "GUI" => {
                        let gui_id = command_options.get("id")
                            .expect("Missing 'id' option required for type 'GUI'!")
                            .to_owned();
                        let sprite_id = command_options.get("sprite")
                            .expect("Missing 'sprite' option required for type 'GUI'!")
                            .to_owned();
                        Transition::SetGUI( gui_id, sprite_id )
                    }
                    _ => panic!("Bad type '{type_of}'!")
                }
            }
            "end" => {
                Transition::End
            }
            _ => panic!("Bad command! {cmd_id}")
        }
    }).collect();

    game_state.transitions_iter = transitions.into_iter();
    game_state.blocking = false;

    info!("Completed pre-compilation");
}
fn run_transitions ( 
    mut character_say_event: EventWriter<CharacterSayEvent>,
    mut emotion_change_event: EventWriter<EmotionChangeEvent>,
    mut background_change_event: EventWriter<BackgroundChangeEvent>,
    mut gui_change_event: EventWriter<GUIChangeEvent>,
    mut gpt_say_event: EventWriter<GPTSayEvent>,
    mut gpt_get_event: EventWriter<GPTGetEvent>,

    mut game_state: ResMut<VisualNovelState>,
) {
    loop {
        while let Some(transition) = game_state.extra_transitions.pop(){
            if game_state.blocking {
                return;
            }
            transition.call(
                &mut character_say_event,
                &mut emotion_change_event,
                &mut background_change_event,
                &mut gui_change_event,
                &mut gpt_say_event,
                &mut gpt_get_event,

                &mut game_state,);
        }
        if game_state.blocking {
            return;
        }
        match game_state.transitions_iter.next() {
            Some(transition) => {
                transition.call(
                    &mut character_say_event,
                    &mut emotion_change_event,
                    &mut background_change_event,
                    &mut gui_change_event,
                    &mut gpt_say_event,
                    &mut gpt_get_event,

                    &mut game_state,);
            },
            None => {
                return;
            }
        }
    }
}

/*
                 _
 _ __ ___   __ _(_)_ __
| '_ ` _ \ / _` | | '_ \
| | | | | | (_| | | | | |
|_| |_| |_|\__,_|_|_| |_|
*/
fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: String::from("Ettethread - Requiem"),
                    resolution: (1280., 800.).into(),
                    present_mode: PresentMode::AutoVsync,
                    // Tells wasm to resize the window according to the available canvas
                    fit_canvas_to_parent: true,
                    // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
                })
        ) 
        .init_resource::<VisualNovelState>()
        .add_startup_system(setup)
        .add_plugin(Compiler)
        .add_plugin(BackgroundController)
        .add_plugin(CharacterController)
        .add_plugin(ChatController)
        .run();
}

fn setup(
    mut commands: Commands,
    mut game_state: ResMut<VisualNovelState>,
) {
    /* Config */
    game_state.playername = String::from("Bolt");
    game_state.api_key = env::var("OPENAI_API_KEY").expect("Environment variable OPENAI_API_KEY needs to be set!");

    /* Basic Scene Setup */
    commands.spawn(Camera2dBundle::default());
}