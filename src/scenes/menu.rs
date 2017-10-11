use scenes::common::*;
use piston_window::*;
use scenes::scene::{Scene, SceneInstance};
use scenes::play::Play;

pub struct Menu {
    next_scene: Option<SceneInstance>
}

impl Menu {
    pub fn new() -> Menu {
        Menu {
            next_scene: None
        }
    }
}

impl Scene for Menu {
    fn get_next(&mut self) -> Option<SceneInstance> {
        self.next_scene.take()
    }

    fn update(&mut self, dt: f64) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, graphics: &mut G2d) -> GameResult<()> {
        clear(WHITE, graphics);

        Ok(())
    }

    fn key_press(&mut self, button: Button) {
        if let Button::Keyboard(key) = button {
            match key {
                Key::Return => {
                    self.next_scene = Some(Box::new(Play::new(None)));
                },
                _ => ()
            }
        }
    }
}