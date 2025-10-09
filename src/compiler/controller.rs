use crate::{compile_to_transitions, BackgroundChangeMessage, CharacterSayMessage, EmotionChangeMessage, GUIChangeMessage, VisualNovelState};

use std::collections::HashMap;
use bevy::prelude::*;
use pest::{iterators::Pair, pratt_parser::PrattParser, Parser};
use pest_derive::Parser;
use anyhow::{anyhow, bail, ensure, Context, Result};

#[derive(Parser)]
#[grammar = "../sabi.pest"]
pub struct SabiParser;

// Create a static PrattParser for expressions
lazy_static::lazy_static! {
    pub static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};

        // Precedence is defined from lowest to highest priority
        PrattParser::new()
            // Highest precedence: addition (+)
            .op(Op::infix(Rule::add, Left))
    };
}

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
    Scene(String),
    End
}
impl Transition {
    fn call(
        &self,
        character_say_message: &mut MessageWriter<CharacterSayMessage>,
        emotion_change_message: &mut MessageWriter<EmotionChangeMessage>,
        background_change_message: &mut MessageWriter<BackgroundChangeMessage>,
        gui_change_message: &mut MessageWriter<GUIChangeMessage>,

        game_state: &mut ResMut<VisualNovelState>,
    ) {
        match self {
            Transition::Say(character_name, msg) => {
                info!("Calling Transition::Say");
                character_say_message.write(CharacterSayMessage {
                    name: character_name.to_owned(),
                    message: msg.to_owned()
                });
                game_state.blocking = true;
            },
            Transition::SetEmotion(character_name, emotion) => {
                info!("Calling Transition::SetEmotion");
                emotion_change_message.write(EmotionChangeMessage {
                    name: character_name.to_owned(),
                    emotion: emotion.to_owned()
                });
            }
            Transition::SetBackground(background_id) => {
                info!("Calling Transition::SetBackground");
                background_change_message.write(BackgroundChangeMessage {
                    background_id: background_id.to_owned()
                });
            },
            Transition::SetGUI(gui_id, sprite_id) => {
                info!("Calling Transition::SetGUI");
                gui_change_message.write(GUIChangeMessage {
                    gui_id: gui_id.to_owned(),
                    sprite_id: sprite_id.to_owned()
                });
            },
            Transition::Log(msg) => println!("{msg}"),
            Transition::Scene(id) => {
                info!("Calling Transition::Scene");
                let script_transitions = game_state.all_script_transitions
                    .get(id.as_str())
                    .expect(&format!("Missing {id} script file! Please remember the game requires an entry.txt script file to have a starting position."))
                    .clone();
                game_state.transitions_iter = script_transitions.into_iter();
                game_state.current_scene_id = id.clone();
            },
            Transition::End => {
                todo!();
            }
        }
    }
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

#[derive(Debug)]
enum Expr {
    Number(f64),
    String(String),
    Add { lhs: Box<Expr>, rhs: Box<Expr> }
}
#[derive(Debug)]
enum CodeStatement {
    Log(Vec<Expr>)
}
#[derive(Debug)]
enum StageCommand {
    BackgroundChange(Box<Expr>)
}
#[derive(Debug)]
struct Dialogue {
    character: String,
    emotion: Option<String>,
    dialogue: String
}
#[derive(Debug)]
enum Statement {
    Code(CodeStatement),
    Stage(StageCommand),
    Dialogue(Dialogue)
}
#[derive(Debug)]
struct Scene {
    id: String,
    statements: Vec<Statement>
}

fn build_expression ( pair: pest::iterators::Pair<Rule> ) -> Result<Expr> {
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::number => {
                let n = primary.as_str().parse::<f64>()?;
                Ok(Expr::Number(n))
            }
            Rule::string => {
                let s = primary.as_str();
                // Remove the surrounding quotes
                let s = &s[1..s.len()-1];
                Ok(Expr::String(s.to_string()))
            },
            Rule::expr => build_expression(primary),
            other => bail!("Unexpected primary expr: {other:?}"),
        })
        .map_infix(|
            left,
            op,
            right
        | match op.as_rule() {
            Rule::add => Ok(Expr::Add {
                lhs: Box::new(left?),
                rhs: Box::new(right?),
            }),
            other => bail!("Unexpected infix operator: {other:?}"),
        })
        .parse(pair.into_inner())
}
fn build_stage_command ( pair: Pair<Rule> ) -> Result<Statement> {
    let pair = pair.into_inner().next()
        .context("Stage command missing command!")?;

    let result = match pair.as_rule() {
        Rule::stage => {
            let mut inner_rules = pair.into_inner();
            let command_pair = inner_rules.next()
                .context("Stage command missing command!")?;
            match command_pair.as_rule() {
                Rule::background_change => {
                    let expr_pair = command_pair.into_inner().next()
                        .context("Background change missing expression!")?;
                    let expr = build_expression(expr_pair)
                        .context("...while building expression for background change")?;
                    StageCommand::BackgroundChange(Box::new(expr))
                },
                other => bail!("Unexpected rule in stage command: {:?}", other)
            }
        },
        other => bail!("Unexpected rule in stage command: {:?}", other)
    };

    Ok(Statement::Stage(result))
}
fn build_code_statement ( pair: Pair<Rule> ) -> Result<Statement> {
    let pair = pair.into_inner().next()
        .context("Code block missing code statement!")?;

    let result = match pair.as_rule() {
        Rule::log => {
            let mut exprs = Vec::new();
            for expr_pair in pair.into_inner() {
                let expr = build_expression(expr_pair)
                    .context("...while building expression for log statement")?;
                exprs.push(expr);
            }
            CodeStatement::Log(exprs)
        },
        other => bail!("Unexpected rule in code statement: {:?}", other)
    };

    Ok(Statement::Code(result))
}
fn build_dialogue ( pair: Pair<Rule> ) -> Result<Statement> {
    ensure!(pair.as_rule() == Rule::dialogue, "Expected dialogue, found {:?}", pair.as_rule());

    let mut inner_rules = pair.into_inner().peekable();
    let character = match inner_rules.next() {
        Some(n) => {
            if n.as_rule() != Rule::character_identifier {
                return Err(anyhow!("Expected character identifier, found {:?}", n.as_rule()).into());
            }
            n.as_str().to_owned()
        },
        None => return Err(anyhow!("Dialogue missing character identifier!").into())
    };
    let emotion = match inner_rules.peek() {
        Some(n) => {
            if n.as_rule() == Rule::emotion_change {
                let emotion_pair = inner_rules.next().unwrap();
                let mut emotion_inner = emotion_pair.into_inner();
                let emotion_name_pair = emotion_inner.next()
                    .context("Emotion change missing emotion name!")?;
                ensure!(emotion_name_pair.as_rule() == Rule::emotion_name, "Expected emotion name, found {:?}", emotion_name_pair.as_rule());
                
                Some(emotion_name_pair.as_str().to_owned())
            } else {
                None
            }
        },
        None => None
    };
    let dialogue_expr_pair = inner_rules.next()
        .context("Dialogue missing dialogue expression!")?;
    let dialogue_expr = build_expression(dialogue_expr_pair)
        .context("...while building dialogue expression")?;
    let dialogue = match dialogue_expr {
        Expr::String(s) => s,
        _ => bail!("Dialogue expression must evaluate to a string!")
    };

    Ok(Statement::Dialogue(Dialogue {
        character,
        emotion,
        dialogue
    }))
}
fn build_scenes ( pair: Pair<Rule> ) -> Result<Vec<Scene>> {
    let mut scenes = Vec::new();

    for scene in pair.into_inner() {
        match scene.as_rule() {
            Rule::scene => {},
            Rule::EOI => continue,
            other => bail!("Unexpected rule when creating scene: {other:?}"),
        }
        
        let mut inner_rules = scene.into_inner();
        let scene_id = {
            let id_pair = inner_rules.next()
                .context("Scene missing ID!")?;
            
            ensure!(id_pair.as_rule() == Rule::scene_num, "Expected scene ID, found {:?}", id_pair.as_rule());

            id_pair.as_str().to_owned()
        };
        let statements = {
            let mut statements = Vec::new();

            for statement in inner_rules {
                let stmt = match statement.as_rule() {
                    Rule::code => build_code_statement(statement).context("...while building code statement")?,
                    Rule::stage => build_stage_command(statement).context("...while building stage command")?,
                    Rule::dialogue => build_dialogue(statement).context("...while building dialogue")?,
                    other => bail!("Unexpected rule when creating scene: {other:?}"),
                };
                statements.push(stmt);
            }
        
            statements
        };
        
        scenes.push(Scene {
            id: scene_id,
            statements
        });
    }

    Ok(scenes)
}

fn pre_compile( mut game_state: ResMut<VisualNovelState>) -> Result<(), BevyError> {
    info!("Starting pre-compilation");

    /* Character Setup */
    // Asset Gathering
    let mut all_script_transitions = HashMap::<String, Vec<Transition>>::new();
    let scripts_dir = std::env::current_dir()
        .context("Failed to get current directory!")?
        .join("assets")
        .join("scripts");
    let scripts_dir_entries: Vec<std::fs::DirEntry> = std::fs::read_dir(scripts_dir)
        .context("Unable to read scripts folder!")?
        .filter_map(|entry_result| {
            entry_result.ok()
        })
        .collect();
    for script_file_entry in scripts_dir_entries {
        let script_name: String = script_file_entry
            .path()
            .as_path()
            .file_stem().context("Your script file must have a name! `.sabi` is illegal.")?
            .to_str().context("Malformed UTF-8 in script file name, please verify it meets UTF-8 validity!")?
            .to_owned();
        let script_contents: String = std::fs::read_to_string(script_file_entry.path())
            .context("Contents of the script file must be valid UTF-8!")?;

        println!("[ Compiling  `{script_contents}` ]");
        
        let scene_pair = SabiParser::parse(Rule::act, &script_contents)
            .context("Failed to parse script file!")?
            .next()
            .context("Script file is empty!")?;
        let scenes_ast: Vec<Scene> = build_scenes(scene_pair)
            .context("...while building scenes from script file")?;
        println!("{scenes_ast:#?}");


        
        //let script_transitions: Vec<Transition> = compile_to_transitions(script_contents);



        println!("[ [ Imported script '{}'! ] ]", script_name);
        //all_script_transitions.insert(script_name, script_transitions);
    }

    // Setup entrypoint
    let entry = all_script_transitions
        .get("entry")
        .context("Missing 'entry' script file! Please remember the game requires an entry.txt script file to have a starting position.")?
        .clone();
    game_state.transitions_iter = entry.into_iter();
    game_state.all_script_transitions = all_script_transitions;
    game_state.current_scene_id = String::from("entry");

    game_state.blocking = false;

    info!("Completed pre-compilation");

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
                transition.call(
                    &mut character_say_message,
                    &mut emotion_change_message,
                    &mut background_change_message,
                    &mut gui_change_message,

                    &mut game_state,);
            }
        }
        if game_state.blocking {
            return;
        }
        match game_state.transitions_iter.next() {
            Some(transition) => {
                transition.call(
                    &mut character_say_message,
                    &mut emotion_change_message,
                    &mut background_change_message,
                    &mut gui_change_message,

                    &mut game_state,);
            },
            None => {
                return;
            }
        }
    }
}
