use std::fs;
use core::slice::Iter;
use std::collections::HashMap;
use regex::Regex;

use nannou::prelude::*;

struct Character<'a> {
    name: &'a str,
    outfit: String,
    emotion: String,
    description: &'a str,
    emotions: Vec<&'a str>,
    xpos: i32,
    ypos: i32,
    scale: f32,
    opacity: f32
}

enum Transition {
    Background(String),
    Say(String, String),
    Log(String),
    End()
}

struct VisualNovel<'a> {
    transitions_iter: Iter<'a, Transition>,
    current_background: String,
}

struct Model {
    backgrounds: HashMap<String, wgpu::Texture>,
}

fn main() {
    let command_structure = Regex::new(r"(\w+)(?: (\w+)\=`(.+?)`)+").unwrap();

    // Compile Script into a vector Transitions, then create an iterator over them
    let full_script_string: String = fs::read_to_string("./assets/scripts/script.txt")
        .expect("Issue reading file!");
    let transitions: Vec<Transition> = full_script_string.lines().map(|line| {
        println!("[ Compiling ] `{line}`");

        let mut command_options: HashMap<String, String> = HashMap::new();

        // Grabs the command in its normal habitat
        let command_captures = command_structure.captures(line).unwrap();

        // Remove the command identifier seperately
        let mut args = command_captures.iter();
        let cmd_id = args
            .nth(1)
            .expect("There should be a command definition.")
            .expect("There should be a match on the first.")
            .as_str();
        println!("CMD: `{cmd_id}`");

        // Adds each option from the command to the options hashmap
        while let Some(capture) = args.next() {
            let option: String = capture.map_or("".to_string(), |m| m.as_str().to_string());
            let value: String  = args.next().expect("Missing value!").map_or("".to_string(), |m| m.as_str().to_string());
            
            command_options.insert(option, value);
        }

        // Try to run the command
        match cmd_id {
            "log" => {
                return Transition::Log(command_options.get("msg").expect("Should have value!").to_string());
            },
            _ => panic!("Bad command! {cmd_id}")
        }
    }).collect();
    let transitions_iter = transitions.iter();

    let primary = VisualNovel {
        transitions_iter,
        current_background: "default".to_string(),
    };

    nannou::app(model)
        .update(update)
        .run();
}

fn model(app: &App) -> Model {
    app.new_window().size(1280, 800).view(view).build().unwrap();

    let assets = app.assets_path().unwrap();

    // Literal Asset Hashmaps
    let mut backgrounds = HashMap::new();


    let background_paths = fs::read_dir(assets.join("images").join("backgrounds"))
        .expect("No backgrounds dir!")
        .map(|entry| entry.unwrap().path());
    for background_path in background_paths {
        let file_name = background_path
            .file_name().unwrap()
            .to_str().unwrap()
            .to_string();
        let file_texture = wgpu::Texture::from_path(app, background_path).unwrap();
        backgrounds.insert(file_name, file_texture);
    }


    
    
    
    Model { backgrounds }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn view(app: &App, model: &Model, frame: Frame) {

    // Prepare to draw.
    let draw = app.draw();

    // Generate sine wave data based on the time of the app
    let sine = app.time.sin();
    let slowersine = (app.time / 2.0).sin();

    // Get boundary of the window (to constrain the movements of our circle)
    let boundary = app.window_rect();

    // Map the sine wave functions to ranges between the boundaries of the window
    let x = map_range(sine, -1.0, 1.0, boundary.left(), boundary.right());
    let y = map_range(slowersine, -1.0, 1.0, boundary.bottom(), boundary.top());

    draw.texture(model.backgrounds.get(primary.current_background).expect("Background does not exist!"));

    // Draw a blue ellipse at the x/y coordinates 0.0, 0.0
    draw.ellipse().color(STEELBLUE).x_y(x, y);

    draw.to_frame(app, &frame).unwrap();
}