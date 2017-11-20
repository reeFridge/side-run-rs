extern crate piston_window;
extern crate byteorder;
extern crate find_folder;
extern crate vecmath;
extern crate cgmath;
extern crate collision;
extern crate button_tracker;
extern crate specs;

#[macro_use]
extern crate conrod;

mod scenes;
mod connection;
mod game_cycle;
mod asset_manager;

use piston_window::{PistonWindow, WindowSettings, TextureSettings, G2dTexture, Flip};
use scenes::ecs_test;
use game_cycle::GameCycle;
use asset_manager::AssetManager;
use std::path::Path;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

pub fn main() {
    let mut window: PistonWindow = WindowSettings::new("side-run", [WIDTH, HEIGHT])
        .resizable(true)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // load assets
    let asset_manager = {
        let buffer_len = WIDTH as usize * HEIGHT as usize;
        let init = vec![128; buffer_len];
        let settings = TextureSettings::new();
        let factory = &mut window.factory;

        let ui_texture_cache = G2dTexture::from_memory_alpha(factory, &init, WIDTH, HEIGHT, &settings).unwrap();
        let player_sprite = G2dTexture::from_path(
            factory,
            Path::new("assets/sprites/player.png"),
            Flip::Horizontal,
            &TextureSettings::new()
        ).unwrap();
        let floor = G2dTexture::from_path(
            factory,
            Path::new("assets/sprites/metal_floor.png"),
            Flip::None,
            &TextureSettings::new()
        ).unwrap();

        let mut manager = AssetManager::new();
        manager.add_texture("ui_cache", ui_texture_cache);
        manager.add_texture("player_sprite", player_sprite);
        manager.add_texture("floor", floor);

        manager
    };
    GameCycle::new(ecs_test::instance(), asset_manager).run(&mut window);
}