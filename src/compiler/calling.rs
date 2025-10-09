use crate::{BackgroundChangeMessage, CharacterSayMessage, EmotionChangeMessage, GUIChangeMessage, VisualNovelState};
use crate::compiler::ast::{CodeStatement, Dialogue, Evaluate, StageCommand, Statement};
use bevy::prelude::*;
use anyhow::{Context, Result};

/* Messages */
#[derive(Message)]
pub struct SceneChangeMessage {
    pub scene_id: String
}

#[derive(Message)]
pub struct ActChangeMessage {
    pub act_id: String
}

pub struct InvokeContext<'l, 'a, 'b, 'c, 'd, 'e, 'f, 'g> {
    pub character_say_message: &'l mut MessageWriter<'a, CharacterSayMessage>,
    pub emotion_change_message: &'l mut MessageWriter<'b, EmotionChangeMessage>,
    pub background_change_message: &'l mut MessageWriter<'c, BackgroundChangeMessage>,
    pub gui_change_message: &'l mut MessageWriter<'d, GUIChangeMessage>,
    pub scene_change_message: &'l mut MessageWriter<'e, SceneChangeMessage>,
    pub act_change_message: &'l mut MessageWriter<'f, ActChangeMessage>,
    pub game_state: &'l mut ResMut<'g, VisualNovelState>
}
pub trait Invoke {
    fn invoke ( &self, ctx: InvokeContext ) -> Result<()>;
}
impl Invoke for Dialogue {
    fn invoke( &self, ctx: InvokeContext ) -> Result<()> {
        let dialogue = self.dialogue.evaluate_into_string()
            .context("...while evaluating Dialogue expression")?;
        info!("Invoking Dialogue::Say");

        ctx.character_say_message.write(CharacterSayMessage {
            name: self.character.to_owned(),
            message: dialogue
        });

        ctx.game_state.blocking = true;

        Ok(())
    }
}
impl Invoke for StageCommand {
    fn invoke( &self, ctx: InvokeContext ) -> Result<()> {
        match self {
            StageCommand::BackgroundChange { background_expr } => {
                let background_id = background_expr.evaluate_into_string()
                    .context("...while evaluating BackgroundChange expression")?;
                
                info!("Invoking StageCommand::BackgroundChange to {}", background_id);
                ctx.background_change_message.write(BackgroundChangeMessage {
                    background_id
                });

                Ok(())
            },
            StageCommand::EmotionChange { character, emotion } => {
                info!("Invoking StageCommand::EmotionChange to {}'s {}", character, emotion);
                ctx.emotion_change_message.write(EmotionChangeMessage {
                    name: character.to_owned(),
                    emotion: emotion.to_owned()
                });

                Ok(())
            },
            StageCommand::GUIChange { id_expr, sprite_expr } => {
                let gui_id = id_expr.evaluate_into_string()
                    .context("...while evaluating GUIChange id expression")?;
                let sprite_id = sprite_expr.evaluate_into_string()
                    .context("...while evaluating GUIChange sprite expression")?;
                
                info!("Invoking StageCommand::GUIChange to {}'s {}", gui_id, sprite_id);
                ctx.gui_change_message.write(GUIChangeMessage {
                    gui_id,
                    sprite_id
                });

                Ok(())
            },
            StageCommand::SceneChange { scene_expr } => {
                let scene_id = scene_expr.evaluate_into_string()
                    .context("...while evaluating SceneChange expression")?;
                
                info!("Invoking StageCommand::SceneChange to {}", scene_id);
                ctx.scene_change_message.write(SceneChangeMessage {
                    scene_id
                });

                Ok(())
            },
            StageCommand::ActChange { act_expr } => {
                let act_id = act_expr.evaluate_into_string()
                    .context("...while evaluating ActChange expression")?;
                
                info!("Invoking StageCommand::ActChange to {}", act_id);
                ctx.act_change_message.write(ActChangeMessage {
                    act_id
                });

                Ok(())
            }
        }
    }
}
impl Invoke for CodeStatement {
    fn invoke( &self, _ctx: InvokeContext ) -> Result<()> {
        match self {
            CodeStatement::Log { exprs } => {
                let mut log_parts: Vec<String> = Vec::new();

                for expr in exprs {
                    let part = expr.evaluate_into_string()
                        .context("...while evaluating Log expression")?;
                    log_parts.push(part);
                }

                let log_message = log_parts.join(" ");
                println!("[ Log ] {}", log_message);

                Ok(())
            },
        }
    }
}
impl Invoke for Statement {
    fn invoke( &self, ctx: InvokeContext ) -> Result<()> {
        Ok(match self {
            Statement::Dialogue(dialogue) => dialogue.invoke(ctx)
                .context("...while invoking Dialogue statement")?,
            Statement::Stage(stage) => stage.invoke(ctx)
                .context("...while invoking StageCommand statement")?,
            Statement::Code(code) => code.invoke(ctx)
                .context("...while invoking Code statement")?,
        })
    }
}