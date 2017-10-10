extern crate piston_window;
extern crate byteorder;

mod scenes;
mod connection;

use piston_window::*;
use scenes::{play, menu};
use scenes::scene::Scene;

struct GameCycle {
    scene: Box<Scene>
}

impl GameCycle {
    fn new(scene: Box<Scene>) -> GameCycle {
        GameCycle { scene: scene }
    }

    fn open_scene(&mut self, scene: Box<Scene>) {
        self.scene = scene;
    }

    fn run(&mut self, window: &mut PistonWindow) {
        loop {
            let may_event = window.next();

            if let Some(event) = may_event {
                event.button(|ButtonArgs { state: button_state, button, .. }| {
                    match button_state {
                        ButtonState::Press => self.scene.key_press(button),
                        _ => ()
                    }
                });

                event.render(|_| {
                    window.draw_2d(&event, |mut ctx, mut graph| {
                        self.scene.draw(&mut ctx, &mut graph).unwrap();
                    });
                });

                event.update(|&UpdateArgs { dt }| {
                    self.scene.update(dt).unwrap();
                });
            } else {
                break;
            }
        }
    }
}

pub fn main() {
    let mut window: PistonWindow = WindowSettings::new("side-run", [800, 600])
        .resizable(true)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let scene = Box::new(play::Play::new(Some("127.0.0.1:7001".to_string())));
    let mut cycle = GameCycle::new(scene);

    cycle.run(&mut window);
}