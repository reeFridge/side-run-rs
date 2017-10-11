use piston_window::*;
use scenes::common::*;

pub type SceneInstance = Box<Scene>;

pub trait Scene {
    fn update(&mut self, dt: f64) -> GameResult<()>;
    fn draw(&mut self, ctx: &mut Context, graphics: &mut G2d) -> GameResult<()>;
    fn key_press(&mut self, button: Button);
    fn get_next(&mut self) -> Option<SceneInstance>;
}