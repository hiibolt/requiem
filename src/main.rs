use std::fs;
use std::collections::HashMap;
use regex::Regex;

use nannou::prelude::*;

struct Model {
    backgrounds: HashMap<String, wgpu::Texture>,
    current_background: String,
}

fn compile(s: &str){
    println!("[ Beginning Compilation ]");

    let mut cmds = s.lines();

    let command_structure = Regex::new(r"(\w+)(?: (\w+)\=`(.+?)`)+").unwrap();

    // Loop through each command
    loop {
        match cmds.next() {
            Some(line) => {
                println!("[ Compiling ] `{line}`");
                let mut command_options: HashMap<&str, &str> = HashMap::new();

                // Grabs the command in its normal habitat
                let command_captures = command_structure.captures(line).unwrap();

                // Rips said command into pieces
                let mut args = command_captures.iter();
                let cmd_id = args
                    .nth(1)
                    .expect("There should be a command definition.")
                    .expect("There should be a match on the first.")
                    .as_str();
                println!("CMD: `{cmd_id}`");

                // Adds each option from the command to the options hashmap
                while let Some(capture) = args.next() {
                    let option: &str = capture.map_or("", |m| m.as_str());
                    let value: &str  = args.next().expect("Missing value!").map_or("", |m| m.as_str());
                    
                    command_options.insert(option, value);
                    println!("Added option `{}` as `{}`", option, value);
                }

                // Try to run the command
                match cmd_id {
                    "log" => {
                        println!("{}", command_options.get("msg").expect("Should have value!"));
                    },
                    _ => panic!("Bad command! {cmd_id}")
                }
            }
            None => {
                println!("Done!");
                return;
            }
        }
    }
}

fn main() {
    println!("Starting Compile - Reading File");

    let contents: &str = &fs::read_to_string("./assets/scripts/script.txt")
        .expect("Issue reading file!");
    println!("[ Contents ]\n{contents}");
    
    nannou::app(model)
        .update(update)
        .run();

    compile(contents);
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
        backgrounds.insert(background_path.file_name().unwrap().to_str().unwrap().to_string(), wgpu::Texture::from_path(app, background_path).unwrap());
    }

    let current_background: String = "idk".to_string();

    Model { backgrounds, current_background }
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

    draw.texture(model.backgrounds.get("main_classroom_day.png").expect("Background does not exist!"));

    // Draw a blue ellipse at the x/y coordinates 0.0, 0.0
    draw.ellipse().color(STEELBLUE).x_y(x, y);

    draw.to_frame(app, &frame).unwrap();
}