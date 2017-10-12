use piston_window::*;
use scenes::common::*;

pub type SceneInstance = Box<Scene>;

pub trait Scene {
    fn update(&mut self, dt: f64) -> GameResult<()>;
    fn draw(&mut self, ctx: &mut Context, graphics: &mut G2d) -> GameResult<()>;
    fn key_press(&mut self, button: Button);
    fn switcher(&mut self) -> &mut Switcher;
}

pub trait Switcher {
    fn get_next(&mut self) -> Option<SceneInstance>;
}

pub struct BaseSwitcher {
    pub next_scene: Option<SceneInstance>
}

impl BaseSwitcher {
    pub fn new(next_scene: Option<SceneInstance>) -> BaseSwitcher {
        BaseSwitcher { next_scene: next_scene }
    }
}

impl Switcher for BaseSwitcher {
    fn get_next(&mut self) -> Option<SceneInstance> {
        self.next_scene.take()
    }
}
