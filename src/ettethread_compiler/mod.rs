use crate::Transition;
use crate::Regex;
use std::collections::HashMap;

pub fn compile_to_transitions(full_script_string: String) -> Vec<Transition> {
    // Build required regets
    let command_structure: Regex = Regex::new(r"(?<cmd_id>\w+)[\s$]").expect("Bad command_structure Regex compilation! Contact the developer.");
    let argument_structure: Regex = Regex::new(r"(?<arg_id>\w+)=`(?<arg_content>[^`]*)`").expect("Bad argument_structure Regex compilation! Contact the developer.");

    full_script_string
        .lines()
        .map(move |line| {
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
        })
        .collect()
}