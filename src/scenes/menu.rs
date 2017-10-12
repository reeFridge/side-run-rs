use scenes::common::*;
use piston_window::*;
use scenes::scene::{Scene, SceneInstance, BaseSwitcher, Switcher};
use scenes::play::Play;

pub struct Menu {
    switcher: BaseSwitcher
}

impl Menu {
    pub fn new() -> Menu {
        Menu {
            switcher: BaseSwitcher::new(None)
        }
    }
}

impl Scene for Menu {
    fn switcher(&mut self) -> &mut Switcher {
        &mut self.switcher
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
                    self.switcher.set_next(Some(Box::new(Play::new(None))));
                },
                _ => ()
            }
        }
    }
}