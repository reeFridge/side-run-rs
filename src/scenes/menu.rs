use scenes::common::*;
use piston_window::*;
use scenes::scene;

pub struct Menu;

impl Menu {
    pub fn new() -> Menu {
        Menu {}
    }
}

impl scene::Scene for Menu {
    fn update(&mut self, dt: f64) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, graphics: &mut G2d) -> GameResult<()> {
        clear(WHITE, graphics);

        Ok(())
    }

    fn key_press(&mut self, button: Button) {
        ()
    }
}