use scenes::common::*;
use scenes::scene::{Scene, SceneInstance, BaseSwitcher, Switcher};
use piston_window::{clear, Context, G2d, Rectangle, rectangle, Transformed, Button, Key};
use cgmath::prelude::*;
use cgmath::{Point2, Vector2};
use collision::prelude::*;
use collision::primitive;
use asset_manager::AssetManager;
use button_tracker::ButtonController;

// specs
//--------------------------------------------------------------------------------------------------
use specs::{Component, DispatcherBuilder, Join, ReadStorage, System, VecStorage, WriteStorage, World, Dispatcher, Fetch};

struct Vel(Vector2<f32>);
impl Component for Vel {
    type Storage = VecStorage<Self>;
}

struct Pos(Point2<f32>);
impl Component for Pos {
    type Storage = VecStorage<Self>;
}

struct Rot(f32);
impl Component for Rot {
    type Storage = VecStorage<Self>;
}

struct BoundingBox(primitive::Rectangle<f32>);

impl BoundingBox {
    fn draw(&self, pos: &Point2<f32>, context: &mut Context, graphics: &mut G2d) {
        let bounding_box = &self.0;

        let bound = bounding_box.get_bound();
        let rect = Rectangle::new_border([1., 1., 0., 1.], 0.5);
        let t = context.transform.trans(pos.x as f64, pos.y as f64);
        let rect_params = rectangle::rectangle_by_corners(
            bound.min().x as f64,
            bound.min().y as f64,
            bound.max().x as f64,
            bound.max().y as f64
        );

        rect.draw(
            rect_params,
            &context.draw_state,
            t,
            graphics
        );
    }
}

impl Component for BoundingBox {
    type Storage = VecStorage<Self>;
}

struct Collides(bool);
impl Component for Collides {
    type Storage = VecStorage<Self>;
}

struct PlayerController {
    speed: f32
}

impl Component for PlayerController {
    type Storage = VecStorage<Self>;
}

struct UpdatePos;

impl<'a> System<'a> for UpdatePos {
    type SystemData = (WriteStorage<'a, Pos>, ReadStorage<'a, Vel>, Fetch<'a, DeltaTime>, Fetch<'a, InputTracker>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut pos, vel, delta, inp) = data;
        let delta = delta.0 as f32;

        for (pos, vel) in (&mut pos, &vel).join() {
            let vel_vec = &vel.0;
            pos.0 += vel_vec * delta;
        }
    }
}

struct UpdateVel;

impl<'a> System<'a> for UpdateVel {
    type SystemData = WriteStorage<'a, Vel>;

    fn run(&mut self, data: Self::SystemData) {
        let mut vel = data;
        let e = 0.5f32;
        let deceleration = 0.85f32;

        for vel in (&mut vel).join() {
            let mut vel_vec = vel.0;

            if !vel_vec.is_zero() {
                let mut new_vec = vel_vec * deceleration;

                if new_vec.magnitude() < e {
                    new_vec = Vector2::zero();
                }

                *vel = Vel(new_vec);
            }
        }
    }
}


struct Movement;

impl<'a> System<'a> for Movement {
    type SystemData = (WriteStorage<'a, Vel>, ReadStorage<'a, PlayerController>, Fetch<'a, InputTracker>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut vel, controller, input) = data;

        let movement_keys = [
            Key::Up,
            Key::Down,
            Key::Left,
            Key::Right
        ];

        for (vel, controller) in (&mut vel, &controller).join() {
            for key in movement_keys.iter() {
                let button = &Button::Keyboard(key.clone());
                let mut vec = Vector2::<f32>::zero();

                if input.0.current_pressed(button) {
                    match key {
                        &Key::Up => vec.y = -1.,
                        &Key::Down => vec.y = 1.,
                        &Key::Left => vec.x = -1.,
                        &Key::Right => vec.x = 1.,
                        _ => ()
                    }
                }

                vel.0 += vec * controller.speed;
            }
        }
    }
}


// Resources
struct DeltaTime(f64);
struct InputTracker(ButtonController);
//--------------------------------------------------------------------------------------------------

pub struct EcsTest<'a, 'b> {
    switcher: BaseSwitcher,
    dispatcher: Dispatcher<'a, 'b>,
    world: World
}

pub fn instance() -> SceneInstance {
    Box::new(EcsTest::new())
}

impl<'a, 'b> EcsTest<'a, 'b> {
    fn new() -> EcsTest<'a, 'b> {
        let mut world = World::new();
        world.register::<Pos>();
        world.register::<Vel>();
        world.register::<Rot>();
        world.register::<Collides>();
        world.register::<BoundingBox>();
        world.register::<PlayerController>();

        world.add_resource(DeltaTime(0.05));
        world.add_resource(InputTracker(ButtonController::new()));

        world.create_entity()
            .with(Vel(Vector2::zero()))
            .with(Pos(Point2::new(20., 20.)))
            .with(BoundingBox(primitive::Rectangle::new(10., 10.)))
            .with(PlayerController { speed: 20. })
            .build();

        let mut dispatcher = DispatcherBuilder::new()
            .add(UpdatePos, "update_pos", &[])
            .add(UpdateVel, "update_vel", &[])
            .add(Movement, "movement", &[])
            .build();

        EcsTest {
            switcher: BaseSwitcher::new(None),
            dispatcher,
            world
        }
    }
}

impl<'a, 'b> Scene for EcsTest<'a, 'b> {
    fn update(&mut self, dt: f64) -> GameResult<()> {
        // This dispatches all the systems in parallel (but blocking).
        self.dispatcher.dispatch(&self.world.res);

        let mut delta = self.world.write_resource::<DeltaTime>();
        *delta = DeltaTime(dt);

        let mut input_tracker = self.world.write_resource::<InputTracker>();
        input_tracker.0.update();

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, graphics: &mut G2d, asset_manager: &mut AssetManager) -> GameResult<()> {
        clear(BLACK, graphics);
        let positions = self.world.read::<Pos>();
        let bounding_boxes = self.world.read::<BoundingBox>();

        for entity in self.world.entities().join() {
            if let (Some(pos), Some(bounding_box)) = (positions.get(entity), bounding_boxes.get(entity)) {
                let pos = &pos.0;
                bounding_box.draw(pos, ctx, graphics);
            }
        }

        Ok(())
    }

    fn key_press(&mut self, button: Button) {
        let mut tracker = self.world.write_resource::<InputTracker>();
        tracker.0.register_press(&button);
    }

    fn key_release(&mut self, button: Button) {
        let mut tracker = self.world.write_resource::<InputTracker>();
        tracker.0.register_release(&button);
    }
    //fn mouse_move(&mut self, _cursor: [f64; 2]) {()}

    fn switcher(&mut self) -> &mut Switcher {
        &mut self.switcher
    }
}