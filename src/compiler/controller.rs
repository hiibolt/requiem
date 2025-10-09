use crate::compiler::calling::{Invoke, InvokeContext};
use crate::{BackgroundChangeMessage, CharacterSayMessage, EmotionChangeMessage, GUIChangeMessage, VisualNovelState};
use crate::compiler::ast::{build_scenes, Acts, Rule, SabiParser};
use std::path::PathBuf;
use bevy::prelude::*;
use anyhow::{bail, ensure, Context, Result};
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



pub struct Compiler;
impl Plugin for Compiler {
    fn build(&self, app: &mut App) {
        app
            .init_state::<RequiemState>()
            .init_resource::<ControllersReady>()
            .add_message::<ControllerReadyMessage>()
            .add_message::<TriggerControllersMessage>()
            .add_systems(Startup, parse)
            .add_systems(Update, check_states.run_if(in_state(RequiemState::WaitingForControllers)))
            .add_systems(Update, run.run_if(in_state(RequiemState::Running)));
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

fn parse_direntry ( 
    acts: &mut Acts,
    dir_entry: std::fs::DirEntry
) -> Result<()> {
    let file_type = dir_entry.file_type()
        .context("Couldn't get file type!")?;
    
    if file_type.is_file() {
        let file_path = dir_entry.path();
        ensure!(file_path.extension().map_or(false, |ext| ext == "sabi"), "Recieved a file that wasn't a `.sabi` file: {:?}", file_path.extension());
        
        // Get the act name from the file stem
        let act_name = dir_entry
            .path()
            .file_stem()
            .context("Invalid script file name")?
            .to_string_lossy()
            .into_owned();
    
        // Compile the act
        info!("Compiling act: {}", act_name);
        let scenes = {
            let script_contents = std::fs::read_to_string(&file_path)
                .with_context(|| format!("Failed to read script file: {:?}", file_path))?;
            let scene_pair = SabiParser::parse(Rule::act, &script_contents)
                .with_context(|| format!("Failed to parse script file: {}", act_name))?
                .next()
                .context("Script file is empty")?;
            
            build_scenes(scene_pair)
                .context("Failed to build scenes from AST")?
        };
        
        ensure!(acts.insert(act_name.clone(), Box::new(scenes)).is_none(), "Duplicate act name '{}'", act_name);
        return Ok(());
    }

    if file_type.is_dir() {
        for entry_result in std::fs::read_dir(dir_entry.path())
            .context("Couldn't read directory!")? {
            let entry = entry_result
                .context("Couldn't get directory entry!")?;
            parse_direntry(acts, entry)?;
        }
        return Ok(());
    }

    bail!("Recieved a directory entry that wasn't a file or directory (likely a symlink)!");
}
fn parse ( mut game_state: ResMut<VisualNovelState> ) -> Result<(), BevyError> {
    info!("Starting parsing");
    
    let mut acts: Acts = Acts::new();
    for dir_entry_result in std::fs::read_dir(PathBuf::from(".").join("assets").join("acts"))
        .context("...while trying to read from the scripts directory")?
    {
        let dir_entry = dir_entry_result
            .context("...while trying to read a directory entry in the scripts directory")?;
        parse_direntry(&mut acts, dir_entry)
            .context("...while trying to parse a script file or directory")?;
    }
    
    // Setup entrypoint
    let act = acts
        .get("1")
        .context("Missing '1' act file! Please ensure you have a `1.sabi` file with SCENE 1.")?
        .clone();
    let scene = act
        .get("1")
        .context("Missing '1' scene in act 1! Please ensure you have a SCENE 1 in your `1.sabi` file.")?
        .clone();
    
    game_state.act = act;
    game_state.scene = scene;
    game_state.statements = game_state.scene.statements.clone().into_iter();
    game_state.blocking = false;
    
    info!("Completed pre-compilation successfully");
    Ok(())
}

fn run<'a, 'b, 'c, 'd, 'e> (
    mut character_say_message: MessageWriter<'a, CharacterSayMessage>,
    mut emotion_change_message: MessageWriter<'b, EmotionChangeMessage>,
    mut background_change_message: MessageWriter<'c, BackgroundChangeMessage>,
    mut gui_change_message: MessageWriter<'d, GUIChangeMessage>,

    mut game_state: ResMut<'e, VisualNovelState>,
) {
    if game_state.blocking {
        return;
    }

    if let Some(statement) = game_state.statements.next() {
        statement.invoke(InvokeContext {
                character_say_message: &mut character_say_message,
                emotion_change_message: &mut emotion_change_message,
                background_change_message: &mut background_change_message,
                gui_change_message: &mut gui_change_message,
                game_state: &mut game_state
            })
            .expect("...while invoking statement");
    }
}
