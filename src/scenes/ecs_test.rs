use scenes::common::*;
use scenes::scene::{Scene, SceneInstance, BaseSwitcher, Switcher};
use piston_window::{clear, Context, G2d, Rectangle, rectangle};
use cgmath::prelude::*;
use cgmath::{Point2, Vector2};
use collision::prelude::*;
use collision::primitive;
use asset_manager::AssetManager;

// specs
//--------------------------------------------------------------------------------------------------
use specs::{Component, DispatcherBuilder, Join, ReadStorage, System, VecStorage, WriteStorage, World, Dispatcher, Fetch};

struct Vel(f32);
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
impl Component for BoundingBox {
    type Storage = VecStorage<Self>;
}

struct Collides(bool);
impl Component for Collides {
    type Storage = VecStorage<Self>;
}

struct UpdatePos;

impl<'a> System<'a> for UpdatePos {
    // These are the resources required for execution.
    // You can also define a struct and `#[derive(SystemData)]`,
    // see the `full` example.
    type SystemData = (WriteStorage<'a, Pos>, ReadStorage<'a, Vel>, Fetch<'a, DeltaTime>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut pos, vel, delta) = data;
        // The `.join()` combines multiple components,
        // so we only access those entities which have
        // both of them.
        // You could also use `par_join()` to get a rayon `ParallelIterator`.
        let delta = delta.0 as f32;

        for (pos, vel) in (&mut pos, &vel).join() {
            pos.0 += Vector2::new(vel.0, vel.0) * delta;
        }
    }
}

struct DeltaTime(f64);
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
        // The `World` is our
        // container for components
        // and other resources.
        let mut world = World::new();
        world.register::<Pos>();
        world.register::<Vel>();
        world.register::<Rot>();
        world.register::<Collides>();
        world.register::<BoundingBox>();

        world.add_resource(DeltaTime(0.05));

        // An entity may or may not contain some component.

        world.create_entity().with(Vel(2.0)).with(Pos(Point2::new(0., 0.))).build();
        world.create_entity().with(Vel(4.0)).with(Pos(Point2::new(0., 0.))).build();
        world.create_entity().with(Vel(1.5)).with(Pos(Point2::new(0., 0.))).build();

        // This entity does not have `Vel`, so it won't be dispatched.
        world.create_entity().with(Pos(Point2::new(0., 0.))).build();

        // This builds a dispatcher.
        // The third parameter of `add` specifies
        // logical dependencies on other systems.
        // Since we only have one, we don't depend on anything.
        // See the `full` example for dependencies.
        let mut dispatcher = DispatcherBuilder::new()
            .add(UpdatePos, "update_pos", &[])
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

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, graphics: &mut G2d, asset_manager: &mut AssetManager) -> GameResult<()> {
        clear(BLACK, graphics);
        let positions = self.world.read::<Pos>();

        for entity in self.world.entities().join() {
            if let Some(pos) = positions.get(entity) {
                let rect = Rectangle::new([1., 0., 1., 1.]);
                let pos = pos.0;

                rect.draw(
                    rectangle::centered_square(pos.x as f64, pos.y as f64, 10.),
                    &ctx.draw_state,
                    ctx.transform.clone(),
                    graphics
                );
            }
        }

        Ok(())
    }

    //fn key_press(&mut self, _button: Button) {()}
    //fn key_release(&mut self, _button: Button) {()}
    //fn mouse_move(&mut self, _cursor: [f64; 2]) {()}

    fn switcher(&mut self) -> &mut Switcher {
        &mut self.switcher
    }
}