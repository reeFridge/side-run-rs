//extern crate ggez;
//extern crate byteorder;
extern crate piston_window;

use piston_window::*;

/*use ggez::conf;
use ggez::Context;
use ggez::event;

mod connection;
mod scenes;

use scenes::play;*/

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

    let mut rotation: f64 = 0.0;

    while let Some(e) = window.next() {
        e.update(|&UpdateArgs { dt }| {
            rotation += 3.0 * dt;
        });

        window.draw_2d(&e, |ctx, graph| {
            clear([0., 0., 0., 1.], graph);
            let center = ctx.transform.trans(400., 300.);
            let square = rectangle::square(0., 0., 100.);
            let red = [1., 0., 0., 1.];
            rectangle(red, square, center.rot_rad(rotation).trans(-50.0, -50.0), graph);
        });
    }
}