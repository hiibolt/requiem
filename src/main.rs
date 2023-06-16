// Iced garbage
use iced::executor;
use iced::widget::canvas::{
    stroke, Cache, Cursor, Geometry, LineCap, Path, Stroke,
};
use iced::widget::{ canvas, container, image, };
use iced::{
    Application, Color, Command, Element, Length, Point, Rectangle, Settings,
    Subscription, Theme, Vector, 
};

use std::fs;
use std::vec::IntoIter;
use std::collections::HashMap;
use regex::Regex;


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

struct VisualNovel {
    backgrounds: HashMap<String, image::Handle>,

    transitions_iter: IntoIter<Transition>,
    current_background: String,
    
    now: time::OffsetDateTime,
    clock: Cache,
}

pub fn main() -> iced::Result {
    VisualNovel::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Tick(time::OffsetDateTime),
}

impl Application for VisualNovel {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        /* WARM UP ASSETS */

        // Literal Asset Hashmaps
        let mut backgrounds = HashMap::new();
        let backgrounds_dir = std::env::current_dir()
            .expect("Failed to get current directory")
            .join("assets")
            .join("backgrounds");
        let background_paths = fs::read_dir(backgrounds_dir)
            .expect("No backgrounds dir!")
            .map(|entry| entry.unwrap().path());
        for background_path in background_paths {
            let file_name = background_path
                .file_name().unwrap()
                .to_str().unwrap()
                .to_string();
            let file_texture = image::Handle::from_path(background_path);

            println!("Imported background '{}'", file_name);
            backgrounds.insert(file_name, file_texture);
        }

        /* PRECOMPILATION */
        let command_structure = Regex::new(r"(\w+)(?: (\w+)\=`(.+?)`)+").unwrap();

        // Compile Script into a vector Transitions, then create an iterator over them
        let full_script_string: String = fs::read_to_string("./assets/scripts/script.txt")
            .expect("Issue reading file!");
        let transitions: Vec<Transition> = full_script_string.lines().map(move |line| {
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

        let visual_novel = VisualNovel {
            backgrounds,

            transitions_iter: transitions.into_iter(),

            current_background: "default".to_string(),

            now: time::OffsetDateTime::now_local()
                .unwrap_or_else(|_| time::OffsetDateTime::now_utc()),
            clock: Default::default(),
        };
        (
            visual_novel,
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Ettethread - Requiem")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Tick(local_time) => {
                let now = local_time;

                if now != self.now {
                    self.now = now;
                    self.clock.clear();
                }
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let canvas = canvas(self as &Self)
            .width(Length::Fill)
            .height(Length::Fill);

        container(canvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(std::time::Duration::from_millis(500)).map(|_| {
            Message::Tick(
                time::OffsetDateTime::now_local()
                    .unwrap_or_else(|_| time::OffsetDateTime::now_utc()),
            )
        })
    }
}

impl<Message> canvas::Program<Message> for VisualNovel {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let clock = self.clock.draw(bounds.size(), |frame| {
            let center = frame.center();
            let radius = frame.width().min(frame.height()) / 2.0;

            let background = Path::circle(center, radius);
            frame.fill(&background, Color::from_rgb8(0x12, 0x93, 0xD8));

            let short_hand =
                Path::line(Point::ORIGIN, Point::new(0.0, -0.5 * radius));

            let long_hand =
                Path::line(Point::ORIGIN, Point::new(0.0, -0.8 * radius));

            let width = radius / 100.0;

            let thin_stroke = || -> Stroke {
                Stroke {
                    width,
                    style: stroke::Style::Solid(Color::WHITE),
                    line_cap: LineCap::Round,
                    ..Stroke::default()
                }
            };

            let wide_stroke = || -> Stroke {
                Stroke {
                    width: width * 3.0,
                    style: stroke::Style::Solid(Color::WHITE),
                    line_cap: LineCap::Round,
                    ..Stroke::default()
                }
            };

            frame.translate(Vector::new(center.x, center.y));

            frame.with_save(|frame| {
                frame.rotate(hand_rotation(self.now.hour(), 12));
                frame.stroke(&short_hand, wide_stroke());
            });

            frame.with_save(|frame| {
                frame.rotate(hand_rotation(self.now.minute(), 60));
                frame.stroke(&long_hand, wide_stroke());
            });

            frame.with_save(|frame| {
                frame.rotate(hand_rotation(self.now.second(), 60));
                frame.stroke(&long_hand, thin_stroke());
            })
        });

        vec![clock]
    }
}

fn hand_rotation(n: u8, total: u8) -> f32 {
    let turns = n as f32 / total as f32;

    2.0 * std::f32::consts::PI * turns
}