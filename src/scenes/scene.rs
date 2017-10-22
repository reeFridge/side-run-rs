use piston_window::*;
use scenes::common::*;

pub type SceneInstance = Box<Scene>;

pub trait Scene {
    fn handle_event(&mut self, _event: Event) {()}
    fn update(&mut self, _dt: f64) -> GameResult<()> {Ok(())}
    fn draw(&mut self, ctx: &mut Context, graphics: &mut G2d) -> GameResult<()>;
    fn key_press(&mut self, _button: Button) {()}
    fn mouse_move(&mut self, _cursor: [f64; 2]) {()}
    fn switcher(&mut self) -> &mut Switcher;
}

pub trait Switcher {
    fn get_next(&mut self) -> Option<SceneInstance>;
}

pub struct BaseSwitcher {
    next_scene: Option<SceneInstance>
}

impl BaseSwitcher {
    pub fn new(scene: Option<SceneInstance>) -> BaseSwitcher {
        BaseSwitcher { next_scene: scene }
    }

    pub fn set_next(&mut self, scene: Option<SceneInstance>) {
        self.next_scene = scene;
    }
}

impl Switcher for BaseSwitcher {
    fn get_next(&mut self) -> Option<SceneInstance> {
        self.next_scene.take()
    }
}
