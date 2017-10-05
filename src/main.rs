extern crate ggez;
extern crate byteorder;

use ggez::conf;
use ggez::Context;
use ggez::event;

mod connection;
mod scenes;

use scenes::play;

pub fn main() {
    let c = conf::Conf::new();
    let ctx = &mut Context::load_from_conf("side_run", "reefridge", c).unwrap();
    let mut state = &mut play::State::new().unwrap();

    match state.connect("127.0.0.1:7001".to_string()) {
        Ok(_) => (),
        Err(e) => println!("Failed to connect: {}", e)
    };

    event::run(ctx, state).unwrap();
}