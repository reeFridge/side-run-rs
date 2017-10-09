extern crate piston_window;
extern crate byteorder;

mod scenes;
mod connection;

use piston_window::*;
use scenes::{play, menu};
use scenes::scene::Scene;

pub fn main() {
    let mut window: PistonWindow = WindowSettings::new("side-run", [800, 600])
        .resizable(true)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut scene_idx = 0 as usize;
    let mut scenes: Vec<Box<Scene>> = vec![
        Box::new(play::Play::new(Some("127.0.0.1:7001".to_string()))),
        Box::new(menu::Menu::new())
    ];

    while let Some(e) = window.next() {
        match e {
            Event::Input(Input::Button(ButtonArgs { state: button_state, button, .. })) => {
                match button_state {
                    ButtonState::Press => {
                        match button {
                            Button::Keyboard(Key::Return) => {
                                scene_idx = match scene_idx {
                                    0 => 1,
                                    1 => 0,
                                    _ => 0
                                };
                            }
                            _ => ()
                        }
                    }
                    _ => ()
                }

                match button_state {
                    ButtonState::Press => scenes[scene_idx].key_press(button),
                    _ => ()
                }
            }
            Event::Loop(Loop::Render(args)) => {
                window.draw_2d(&e, |mut ctx, mut graph| {
                    scenes[scene_idx].draw(&mut ctx, &mut graph).unwrap();
                });
            }
            Event::Loop(Loop::Update(UpdateArgs { dt })) => {
                scenes[scene_idx].update(dt).unwrap();
            }
            _ => ()
        };
    }
}