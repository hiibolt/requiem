use pest::{iterators::Pair, pratt_parser::PrattParser};
use pest_derive::Parser;
use anyhow::{bail, ensure, Context, Result};

#[derive(Parser)]
#[grammar = "../sabi.pest"]
pub struct SabiParser;

lazy_static::lazy_static! {
    pub static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};
        // Precedence is defined from lowest to highest priority
        PrattParser::new()
            .op(Op::infix(Rule::add, Left))
    };
}

// Trait for evaluating expressions by flattening them
pub trait Evaluate {
    fn evaluate(&self) -> Result<Expr>;
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    String(String),
    Add { lhs: Box<Expr>, rhs: Box<Expr> }
}

impl Evaluate for Expr {
    fn evaluate(&self) -> Result<Expr> {
        match self {
            Expr::String(_) | Expr::Number(_) => Ok(self.clone()),
            Expr::Add { lhs, rhs } => {
                let left = lhs.evaluate().context("Failed to evaluate left side of addition")?;
                let right = rhs.evaluate().context("Failed to evaluate right side of addition")?;
                
                match (&left, &right) {
                    (Expr::Number(l), Expr::Number(r)) => {
                        Ok(Expr::Number(l + r))
                    },
                    (Expr::String(l), Expr::String(r)) => {
                        Ok(Expr::String(format!("{}{}", l, r)))
                    },
                    (Expr::Number(n), Expr::String(s)) => {
                        Ok(Expr::String(format!("{}{}", n, s)))
                    },
                    (Expr::String(s), Expr::Number(n)) => {
                        Ok(Expr::String(format!("{}{}", s, n)))
                    },
                    _ => {
                        // For complex expressions, convert to strings and concatenate
                        let left_str = expr_to_string(&left)?;
                        let right_str = expr_to_string(&right)?;
                        Ok(Expr::String(format!("{}{}", left_str, right_str)))
                    }
                }
            }
        }
    }
}

impl Evaluate for CodeStatement {
    fn evaluate(&self) -> Result<Expr> {
        match self {
            CodeStatement::Log(exprs) => {
                let mut parts = Vec::new();
                for expr in exprs {
                    let evaluated = expr.evaluate()?;
                    parts.push(expr_to_string(&evaluated)?);
                }
                Ok(Expr::String(parts.join(" ")))
            }
        }
    }
}

// Helper function to convert Expr to String
pub fn expr_to_string(expr: &Expr) -> Result<String> {
    match expr {
        Expr::String(s) => Ok(s.clone()),
        Expr::Number(n) => Ok(n.to_string()),
        Expr::Add { .. } => {
            let evaluated = expr.evaluate()?;
            expr_to_string(&evaluated)
        }
    }
}

// Shorthand function to evaluate an expression and convert to string
pub fn evaluate_into_string(expr: &Expr) -> Result<String> {
    let evaluated = expr.evaluate()
        .context("Failed to evaluate expression")?;
    expr_to_string(&evaluated)
        .context("Failed to convert evaluated expression to string")
}

// Shorthand function for code statements to evaluate into string
pub fn evaluate_code_into_string(code_stmt: &CodeStatement) -> Result<String> {
    let evaluated = code_stmt.evaluate()
        .context("Failed to evaluate code statement")?;
    expr_to_string(&evaluated)
        .context("Failed to convert evaluated code statement to string")
}

#[derive(Debug, Clone)]
pub enum CodeStatement {
    Log(Vec<Expr>)
}

#[derive(Debug, Clone)]
pub enum StageCommand {
    BackgroundChange(Box<Expr>),
    GUIChange { id: Box<Expr>, sprite: Box<Expr> }
}

#[derive(Debug, Clone)]
pub struct Dialogue {
    pub character: String,
    pub emotion: Option<String>,
    pub dialogue: String
}

#[derive(Debug, Clone)]
pub enum Statement {
    Code(CodeStatement),
    Stage(StageCommand),
    Dialogue(Dialogue)
}

#[derive(Debug, Clone)]
pub struct Scene {
    pub id: String,
    pub statements: Vec<Statement>
}

pub fn build_expression(pair: pest::iterators::Pair<Rule>) -> Result<Expr> {
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::number => {
                primary.as_str().parse::<f64>()
                    .map(Expr::Number)
                    .context("Failed to parse number")
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
        .map_infix(|left, op, right| {
            match op.as_rule() {
                Rule::add => Ok(Expr::Add {
                    lhs: Box::new(left.context("Failed to evaluate left operand")?),
                    rhs: Box::new(right.context("Failed to evaluate right operand")?),
                }),
                other => bail!("Unexpected infix operator: {other:?}"),
            }
        })
        .parse(pair.into_inner())
        .context("Failed to parse expression")
}

pub fn build_stage_command(pair: Pair<Rule>) -> Result<Statement> {
    ensure!(pair.as_rule() == Rule::stage, 
        "Expected stage rule, found {:?}", pair.as_rule());
    
    let command_pair = pair.into_inner().next()
        .context("Stage command missing inner command")?;
    
    let result = match command_pair.as_rule() {
        Rule::background_change => {
            let expr_pair = command_pair.into_inner().next()
                .context("Background change missing expression")?;
            let expr = build_expression(expr_pair)
                .context("Failed to build expression for background change")?;
            StageCommand::BackgroundChange(Box::new(expr))
        },
        Rule::gui_change => {
            let mut inner = command_pair.into_inner();
            let gui_element_pair = inner.next()
                .context("GUI change missing GUI element")?;
            let sprite_expr_pair = inner.next()
                .context("GUI change missing sprite expression")?;
            
            // Convert gui_element to the appropriate ID
            let gui_id = match gui_element_pair.as_str() {
                "textbox" => "_textbox_background",
                "namebox" => "_namebox_background",
                other => bail!("Unknown GUI element: {}", other)
            };
            
            let sprite_expr = build_expression(sprite_expr_pair)
                .context("Failed to build sprite expression for GUI change")?;
            
            StageCommand::GUIChange { 
                id: Box::new(Expr::String(gui_id.to_string())), 
                sprite: Box::new(sprite_expr) 
            }
        },
        other => bail!("Unexpected rule in stage command: {:?}", other)
    };
    
    Ok(Statement::Stage(result))
}

pub fn build_code_statement(code_pair: Pair<Rule>) -> Result<Statement> {
    ensure!(code_pair.as_rule() == Rule::code, 
        "Expected code rule, found {:?}", code_pair.as_rule());
    
    let statement_pair = code_pair.into_inner().next()
        .context("Code block missing statement")?;
    
    let result = match statement_pair.as_rule() {
        Rule::log => {
            let mut exprs = Vec::new();
            for expr_pair in statement_pair.into_inner() {
                let expr = build_expression(expr_pair)
                    .context("Failed to build expression for log statement")?;
                exprs.push(expr);
            }
            CodeStatement::Log(exprs)
        },
        other => bail!("Unexpected rule in code statement: {:?}", other)
    };
    
    Ok(Statement::Code(result))
}

pub fn build_dialogue(pair: Pair<Rule>) -> Result<Statement> {
    ensure!(pair.as_rule() == Rule::dialogue, 
        "Expected dialogue, found {:?}", pair.as_rule());
    
    let mut inner_rules = pair.into_inner().peekable();
    
    let character = inner_rules.next()
        .context("Dialogue missing character identifier")?
        .as_str()
        .to_owned();
    
    let emotion = match inner_rules.peek() {
        Some(n) if n.as_rule() == Rule::emotion_change => {
            let emotion_pair = inner_rules.next().unwrap();
            let emotion_name_pair = emotion_pair.into_inner().next()
                .context("Emotion change missing emotion name")?;
            
            ensure!(emotion_name_pair.as_rule() == Rule::emotion_name, 
                "Expected emotion name, found {:?}", emotion_name_pair.as_rule());
            
            Some(emotion_name_pair.as_str().to_owned())
        },
        _ => None
    };
    
    let dialogue_expr_pair = inner_rules.next()
        .context("Dialogue missing dialogue expression")?;
    
    let dialogue_expr = build_expression(dialogue_expr_pair)
        .context("Failed to build dialogue expression")?;
    
    let evaluated_expr = dialogue_expr.evaluate()
        .context("Failed to evaluate dialogue expression")?;
    
    let dialogue = expr_to_string(&evaluated_expr)
        .context("Failed to convert dialogue expression to string")?;
    
    Ok(Statement::Dialogue(Dialogue {
        character,
        emotion,
        dialogue
    }))
}

pub fn build_scenes(pair: Pair<Rule>) -> Result<Vec<Scene>> {
    let mut scenes = Vec::new();
    
    for scene_pair in pair.into_inner() {
        match scene_pair.as_rule() {
            Rule::scene => {
                let mut inner_rules = scene_pair.into_inner();
                
                let scene_id = inner_rules.next()
                    .context("Scene missing ID")?
                    .as_str()
                    .to_owned();
                
                let mut statements = Vec::new();
                for statement_pair in inner_rules {
                    let stmt = match statement_pair.as_rule() {
                        Rule::code => build_code_statement(statement_pair)
                            .context("Failed to build code statement")?,
                        Rule::stage => build_stage_command(statement_pair)
                            .context("Failed to build stage command")?,
                        Rule::dialogue => build_dialogue(statement_pair)
                            .context("Failed to build dialogue")?,
                        other => bail!("Unexpected rule in scene: {:?}", other),
                    };
                    statements.push(stmt);
                }
                
                scenes.push(Scene {
                    id: scene_id,
                    statements
                });
            },
            Rule::EOI => continue,
            other => bail!("Unexpected rule when parsing scenes: {:?}", other),
        }
    }
    
    Ok(scenes)
}