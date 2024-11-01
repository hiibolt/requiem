use crate::info;
use crate::Character;
use crate::VisualNovelState;
use crate::Transition;

use regex::Regex;
use serde::{ Serialize, Deserialize };



/* Events */
pub struct GPTGetEvent {
    pub past_character: String,
    pub past_goal: String
}
pub struct GPTSayEvent {
    pub name: String,
    pub goal: String,
    pub advice: Option<String>
}
pub struct CharacterSayEvent {
    pub name: String,
    pub message: String
}
pub struct GUIChangeEvent {
    pub gui_id: String,
    pub sprite_id: String
}

/* Custom Types */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String
}
#[derive(Deserialize, Debug)]
pub struct ChatChoice {
    //index: usize,
    pub message: Message,
    //finish_reason: String
}
#[derive(Deserialize, Debug)]
pub struct CompletionChoice {
    //index: usize,
    pub text: String,
    //finish_reason: String
}
#[derive(Deserialize, Debug)]
pub struct Usage {
    //prompt_tokens: usize,
    //completion_tokens: usize,
    pub total_tokens: usize
}
#[derive(Deserialize, Debug)]
pub struct ChatResponse {
    //id: Option<String>,
    //object: Option<String>,
    //created: Option<u64>,
    //model: Option<String>,
    choices: Vec<ChatChoice>,
    usage: Option<Usage>
}
#[derive(Deserialize, Debug)]
pub struct CompletionResponse {
    //id: Option<String>,
    //object: Option<String>,
    //created: Option<u64>,
    //model: Option<String>,
    choices: Vec<CompletionChoice>,
    usage: Option<Usage>
}
#[derive(Serialize, Debug)]
pub struct GPTTurboRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32
}
#[derive(Serialize, Debug)]
pub struct CompletionRequest {
    model: String,
    prompt: String,
    temperature: f32,
    max_tokens: usize
}
#[derive(Deserialize)]
pub struct GoalResponse {
    reason: Option<String>,
    goal_status: String,
    advice: Option<String>,
}

#[derive(Debug)]
pub enum GPTError {
    RequestBuilderError,
    LengthError,
    IOError,
    OpenAIError,
    UnparseableOpenAIResponse,
    Null
}
pub fn message_context_to_stringified_request(character: &Character, game_state: &VisualNovelState, event: &GPTSayEvent) -> Result<String, GPTError>{
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
pub fn query_gpt_turbo(request_string: &String, api_key: &String) -> Result<String, GPTError> {
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
pub fn generate_chat_transitions(character: &Character, game_state: &VisualNovelState, event: &GPTSayEvent) -> Result<Vec<Transition>, GPTError> {
    let mut ret = Vec::<Transition>::new();

    // Serialize the request
    let serialized_request: String = message_context_to_stringified_request(character, game_state, event)?;

    // Make the request
    println!("[ Sending GPT request to OpenAI ]");
    let response_result_object: ChatResponse = query_gpt_turbo(&serialized_request, &game_state.api_key)
        .and_then(|response| {
            serde_json::from_str(&response).or(Err(GPTError::UnparseableOpenAIResponse))
        })?;

    // Parse the response
    let response_message = response_result_object.choices[0].message.content.clone();

    println!("[ Response: {} ]", response_message);
    if let Some(usage) = response_result_object.usage {
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
    Ok(ret)
}
pub fn determine_goal_status(character: &Character, game_state: &VisualNovelState, event: &GPTSayEvent) -> Option<bool> {
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