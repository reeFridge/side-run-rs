extern crate piston_window;
extern crate byteorder;
mod scenes;
mod connection;

use piston_window::*;
use scenes::play;

pub fn main() {
    let mut window: PistonWindow = WindowSettings::new("side-run", [800, 600])
        .resizable(true)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut state = play::State::new().unwrap();

    match state.connect("127.0.0.1:7001".to_string()) {
        Ok(_) => (),
        Err(e) => println!("Failed to connect: {}", e)
    };

    while let Some(e) = window.next() {
        e.update(|&UpdateArgs { dt }| {
            state.update(dt).unwrap();
        });

        match e {
            Event::Input(Input::Button(ButtonArgs { state: button_state, button, .. })) => {
                match button_state {
                    ButtonState::Press => state.key_press(button),
                    _ => ()
                }
            },
            _ => ()
        };

        window.draw_2d(&e, |mut ctx, mut graph| {
            state.draw(&mut ctx,&mut graph).unwrap();
        });
    }
}