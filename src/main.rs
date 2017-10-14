extern crate piston_window;
extern crate byteorder;
extern crate conrod;

mod scenes;
mod connection;
mod game_cycle;

use piston_window::*;
use scenes::play::Play;
use game_cycle::GameCycle;

pub fn main() {
    let mut window: PistonWindow = WindowSettings::new("side-run", [800, 600])
        .resizable(true)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let scene = Box::new(Play::new(Some("127.0.0.1:7001".to_string())));
    let mut cycle = GameCycle::new(scene);

    cycle.run(&mut window);
}