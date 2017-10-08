extern crate piston_window;
mod scenes;

use piston_window::*;
use scenes::play;

pub fn main() {
/*    let c = conf::Conf::new();
    let ctx = &mut Context::load_from_conf("side_run", "reefridge", c).unwrap();
    let mut state = &mut play::State::new().unwrap();

    match state.connect("127.0.0.1:7001".to_string()) {
        Ok(_) => (),
        Err(e) => println!("Failed to connect: {}", e)
    };

    event::run(ctx, state).unwrap();*/

    let mut window: PistonWindow = WindowSettings::new("side-run", [800, 600])
        .resizable(true)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut state = play::State::new().unwrap();

    while let Some(e) = window.next() {
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
            state.draw(&mut ctx,&mut graph);
        });
    }
}