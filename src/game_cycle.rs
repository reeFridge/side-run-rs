use piston_window::*;
use scenes::scene::SceneInstance;
use asset_manager::AssetManager;

pub struct GameCycle {
    scene: SceneInstance,
    asset_manager: AssetManager
}

impl GameCycle {
    pub fn new(scene: SceneInstance, asset_manager: AssetManager) -> GameCycle {
        GameCycle { scene, asset_manager }
    }

    pub fn run(&mut self, window: &mut PistonWindow) {
        loop {
            let may_event = window.next();

            if let Some(event) = may_event {
                self.scene.handle_event(event.clone());

                event.mouse_cursor(|x, y| {
                    self.scene.mouse_move([x, y]);
                });

                event.button(|ButtonArgs { state: button_state, button, .. }| {
                    match button_state {
                        ButtonState::Press => self.scene.key_press(button),
                        ButtonState::Release => self.scene.key_release(button),
                        _ => ()
                    }
                });

                event.render(|_| {
                    window.draw_2d(&event, |mut ctx, mut graph| {
                        self.scene.draw(&mut ctx, &mut graph, &mut self.asset_manager).unwrap();
                    });
                });

                event.update(|&UpdateArgs { dt }| {
                    self.scene.update(dt).unwrap();
                });

                self.scene.switcher()
                    .get_next()
                    .and_then(|next_scene| Some(self.set_scene(next_scene)));
            } else {
                break;
            }
        }
    }

    fn set_scene(&mut self, scene: SceneInstance) {
        self.scene = scene;
    }
}
