use crate::{BackgroundChangeMessage, CharacterSayMessage, EmotionChangeMessage, GUIChangeMessage, VisualNovelState};
use crate::compiler::ast::{self, build_scenes, Evaluate, Rule, SabiParser};
use std::collections::HashMap;
use bevy::prelude::*;
use anyhow::{Context, Result};
use pest::Parser;

/* States */
#[derive(States, Debug, Default, Clone, Copy, Hash, Eq, PartialEq)]
enum RequiemState {
    #[default]
    WaitingForControllers,
    Running,
}

#[derive(Resource, Default)]
struct ControllersReady {
    pub background_controller: bool,
    pub character_controller: bool,
    pub chat_controller: bool,
}

/* Messages */
#[derive(Message)]
pub struct TriggerControllersMessage;
#[derive(Message)]
pub struct ControllerReadyMessage(pub Controller);

/* Custom Types */
pub enum Controller {
    Background,
    Character,
    Chat,
}

#[derive(Clone, Debug)]
pub enum Transition {
    Say(String, String),
    SetEmotion(String, String),
    SetBackground(String),
    SetGUI(String, String),
    Log(String),
    Scene(String)
}

impl Transition {
    fn call(
        &self,
        character_say_message: &mut MessageWriter<CharacterSayMessage>,
        emotion_change_message: &mut MessageWriter<EmotionChangeMessage>,
        background_change_message: &mut MessageWriter<BackgroundChangeMessage>,
        gui_change_message: &mut MessageWriter<GUIChangeMessage>,
        game_state: &mut ResMut<VisualNovelState>,
    ) -> Result<()> {
        match self {
            Transition::Say(character_name, msg) => {
                info!("Calling Transition::Say");
                character_say_message.write(CharacterSayMessage {
                    name: character_name.to_owned(),
                    message: msg.to_owned()
                });
                game_state.blocking = true;
                Ok(())
            },
            Transition::SetEmotion(character_name, emotion) => {
                info!("Calling Transition::SetEmotion");
                emotion_change_message.write(EmotionChangeMessage {
                    name: character_name.to_owned(),
                    emotion: emotion.to_owned()
                });
                Ok(())
            }
            Transition::SetBackground(background_id) => {
                info!("Calling Transition::SetBackground");
                background_change_message.write(BackgroundChangeMessage {
                    background_id: background_id.to_owned()
                });
                Ok(())
            },
            Transition::SetGUI(gui_id, sprite_id) => {
                info!("Calling Transition::SetGUI");
                gui_change_message.write(GUIChangeMessage {
                    gui_id: gui_id.to_owned(),
                    sprite_id: sprite_id.to_owned()
                });
                Ok(())
            },
            Transition::Log(msg) => {
                println!("{msg}");
                Ok(())
            },
            Transition::Scene(id) => {
                info!("Calling Transition::Scene");
                let script_transitions = game_state.all_script_transitions
                    .get(id.as_str())
                    .with_context(|| format!("Missing script file for scene: {}", id))?
                    .clone();
                game_state.transitions_iter = script_transitions.into_iter();
                game_state.current_scene_id = id.clone();
                Ok(())
            }
        }
    }
}

// Convert AST scenes to transitions using the Evaluate trait
fn scenes_to_transitions(scenes: Vec<ast::Scene>) -> Result<HashMap<String, Vec<Transition>>> {
    let mut all_transitions = HashMap::new();
    
    for scene in scenes {
        let mut transitions = Vec::new();
        
        for statement in scene.statements {
            match statement {
                ast::Statement::Code(code_stmt) => {
                    match code_stmt {
                        ast::CodeStatement::Log(exprs) => {
                            let log_message = {
                                let mut body = String::new();
                                for expr in exprs {
                                    let evaluated = expr.evaluate_into_string()
                                        .context("Failed to evaluate log expression")?;

                                    body.push_str(&evaluated);
                                }
                                body
                            };
                            
                            transitions.push(Transition::Log(log_message));
                        }
                    }
                },
                ast::Statement::Stage(stage_cmd) => {
                    match stage_cmd {
                        ast::StageCommand::BackgroundChange(expr) => {
                            let background_id = expr.evaluate_into_string()
                                .context("Failed to evaluate background change expression")?;
                            transitions.push(Transition::SetBackground(background_id));
                        },
                        ast::StageCommand::GUIChange { id, sprite } => {
                            let gui_id = id.evaluate_into_string()
                                .context("Failed to evaluate GUI ID expression")?;
                            let sprite_id = sprite.evaluate_into_string()
                                .context("Failed to evaluate GUI sprite expression")?;
                            transitions.push(Transition::SetGUI(gui_id, sprite_id));
                        },
                        ast::StageCommand::EmotionChange { character, emotion } => {
                            transitions.push(Transition::SetEmotion(character, emotion.to_uppercase()));
                        }
                    }
                },
                ast::Statement::Dialogue(dialogue) => {
                    let dialogue_text = dialogue.dialogue.evaluate_into_string()
                        .context("Failed to evaluate dialogue expression")?;
                    transitions.push(Transition::Say(dialogue.character, dialogue_text));
                }
            }
        }
        
        all_transitions.insert(scene.id, transitions);
    }
    
    Ok(all_transitions)
}

pub struct Compiler;
impl Plugin for Compiler {
    fn build(&self, app: &mut App) {
        app
            .init_state::<RequiemState>()
            .init_resource::<ControllersReady>()
            .add_message::<ControllerReadyMessage>()
            .add_message::<TriggerControllersMessage>()
            .add_systems(Startup, pre_compile)
            .add_systems(Update, check_states.run_if(in_state(RequiemState::WaitingForControllers)))
            .add_systems(Update, run_transitions.run_if(in_state(RequiemState::Running)));
    }
}

fn check_states(
    mut msg_controller_reader: MessageReader<ControllerReadyMessage>,
    mut controllers_state: ResMut<ControllersReady>,
    mut msg_writer: MessageWriter<TriggerControllersMessage>,
    mut requiem_state: ResMut<NextState<RequiemState>>,
) {
    for event in msg_controller_reader.read() {
        let controller = match event.0 {
            Controller::Background => &mut controllers_state.background_controller,
            Controller::Character => &mut controllers_state.character_controller,
            Controller::Chat => &mut controllers_state.chat_controller,
        };
        *controller = true;
    }
    if controllers_state.background_controller
       && controllers_state.character_controller
       && controllers_state.chat_controller {
        msg_writer.write(TriggerControllersMessage);
        requiem_state.set(RequiemState::Running);
    }
}

fn pre_compile( mut game_state: ResMut<VisualNovelState> ) -> Result<(), BevyError> {
    info!("Starting pre-compilation");
    
    let mut all_script_transitions = HashMap::<String, Vec<Transition>>::new();
    let scripts_dir = std::env::current_dir()
        .context("Failed to get current directory")?
        .join("assets")
        .join("scripts");
    
    let scripts_dir_entries: Vec<std::fs::DirEntry> = std::fs::read_dir(&scripts_dir)
        .with_context(|| format!("Unable to read scripts folder: {:?}", scripts_dir))?
        .collect::<std::io::Result<Vec<_>>>()
        .context("Failed to read directory entries")?;
    
    for script_file_entry in scripts_dir_entries {
        let file_path = script_file_entry.path();
        
        // Only process .sabi files
        if file_path.extension().map_or(true, |ext| ext != "sabi") {
            continue;
        }
        
        let script_name = file_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .with_context(|| format!("Invalid script file name: {:?}", file_path))?
            .to_string();
        
        let script_contents = std::fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read script file: {:?}", file_path))?;
        
        info!("Compiling script: {}", script_name);
        
        let transitions_map = {
            let mut parsed = SabiParser::parse(Rule::act, &script_contents)
                .with_context(|| format!("Failed to parse script file: {}", script_name))?;
            let scenes_ast = parsed.next()
                .context("Script file is empty")
                .and_then(build_scenes)
                .context("Failed to build scenes from AST")?;
            scenes_to_transitions(scenes_ast)
                .context("Failed to convert scenes to transitions")?
        };
        
        for (scene_id, transitions) in transitions_map {
            let full_scene_id = if script_name == "entry" && scene_id == "1" {
                "entry".to_string()
            } else {
                format!("{}_{}", script_name, scene_id)
            };
            all_script_transitions.insert(full_scene_id, transitions);
        }
        
        info!("Successfully imported script '{}'", script_name);
    }
    
    // Setup entrypoint
    let entry = all_script_transitions.get("entry")
        .context("Missing 'entry' script file! Please ensure you have an entry.sabi file with SCENE 1.")?
        .clone();
    
    game_state.transitions_iter = entry.into_iter();
    game_state.all_script_transitions = all_script_transitions;
    game_state.current_scene_id = String::from("entry");
    game_state.blocking = false;
    
    info!("Completed pre-compilation successfully");
    Ok(())
}

fn run_transitions (
    mut character_say_message: MessageWriter<CharacterSayMessage>,
    mut emotion_change_message: MessageWriter<EmotionChangeMessage>,
    mut background_change_message: MessageWriter<BackgroundChangeMessage>,
    mut gui_change_message: MessageWriter<GUIChangeMessage>,

    mut game_state: ResMut<VisualNovelState>,
) {
    loop {
        while game_state.extra_transitions.len() > 0 {
            if game_state.blocking {
                return;
            }
            if let Some(transition) = game_state.extra_transitions.pop() {
                if let Err(e) = transition.call(
                    &mut character_say_message,
                    &mut emotion_change_message,
                    &mut background_change_message,
                    &mut gui_change_message,
                    &mut game_state,
                ) {
                    error!("Failed to execute transition: {:?}", e);
                    // Continue processing other transitions instead of crashing
                }
            }
        }
        if game_state.blocking {
            return;
        }
        match game_state.transitions_iter.next() {
            Some(transition) => {
                if let Err(e) = transition.call(
                    &mut character_say_message,
                    &mut emotion_change_message,
                    &mut background_change_message,
                    &mut gui_change_message,
                    &mut game_state,
                ) {
                    error!("Failed to execute transition: {:?}", e);
                    // Continue processing other transitions instead of crashing
                }
            },
            None => {
                return;
            }
        }
    }
}
