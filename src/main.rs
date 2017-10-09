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
    let mut scene: Box<Scene> = Box::new(play::Play::new(Some("127.0.0.1:7001".to_string())));

    while let Some(e) = window.next() {
        match e {
            Event::Input(Input::Button(ButtonArgs { state: button_state, button, .. })) => {
                match button_state {
                    ButtonState::Press => {
                        match button {
                            Button::Keyboard(Key::Return) => {
                                scene = Box::new(menu::Menu::new());
                            }
                            _ => ()
                        }
                    }
                    _ => ()
                }

                match button_state {
                    ButtonState::Press => scene.key_press(button),
                    _ => ()
                }
            }
            Event::Loop(Loop::Render(args)) => {
                window.draw_2d(&e, |mut ctx, mut graph| {
                    scene.draw(&mut ctx, &mut graph).unwrap();
                });
            }
            Event::Loop(Loop::Update(UpdateArgs { dt })) => {
                scene.update(dt).unwrap();
            }
            _ => ()
        };
    }
}