extern crate piston_window;
extern crate byteorder;
extern crate find_folder;

#[macro_use]
extern crate conrod;

mod scenes;
mod connection;
mod game_cycle;

use piston_window::*;
use scenes::menu::Menu;
use game_cycle::GameCycle;

pub fn main() {
    let mut window: PistonWindow = WindowSettings::new("side-run", [800, 600])
        .resizable(true)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let texture = {
        const WIDTH: u32 = 800;
        const HEIGHT: u32 = 600;
        let buffer_len = WIDTH as usize * HEIGHT as usize;
        let init = vec![128; buffer_len];
        let settings = TextureSettings::new();
        let factory = &mut window.factory;

        G2dTexture::from_memory_alpha(factory, &init, WIDTH, HEIGHT, &settings).unwrap()
    };

    let scene = Box::new(Menu::new(texture));
    let mut cycle = GameCycle::new(scene);

    cycle.run(&mut window);
}