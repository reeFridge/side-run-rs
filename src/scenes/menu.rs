use scenes::common::*;
use piston_window::*;
use scenes::scene::{Scene, SceneInstance, BaseSwitcher, Switcher};
use scenes::play::Play;
use conrod;

pub struct Menu {
    switcher: BaseSwitcher,
    ui: conrod::Ui
}

impl Menu {
    pub fn new() -> Menu {
        Menu {
            switcher: BaseSwitcher::new(None),
            ui: conrod::UiBuilder::new([800., 600.]).build()
        }
    }
}

impl Scene for Menu {
    fn handle_event(&mut self, event: Event) {
        if let Some(e) = conrod::backend::piston::event::convert(event, 800. as conrod::Scalar,  600. as conrod::Scalar) {
            self.ui.handle_event(e);
        }
    }

    fn switcher(&mut self) -> &mut Switcher {
        &mut self.switcher
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