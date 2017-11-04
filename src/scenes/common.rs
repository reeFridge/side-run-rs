use piston_window::*;
use piston_window::math::*;

pub type GameResult<T> = Result<T, String>;

pub trait Movable {
    fn translate_by_direction(&mut self, direction: Direction, units: f64);
}

impl Movable for Vec2d {
    fn translate_by_direction(&mut self, direction: Direction, units: f64) {
        let translate_vec = mul_scalar(Vec2d::from(direction), units);
        *self = add(translate_vec, self.clone());
    }
}

// TODO: forward, back, left, right
#[derive(Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    Stay
}

impl From<Direction> for Vec2d {
    fn from(dir: Direction) -> Self {
        match dir {
            Direction::Left => Vec2d::from([-1., 0.]),
            Direction::Right => Vec2d::from([1., 0.]),
            Direction::Up => Vec2d::from([0., -1.]),
            Direction::Down => Vec2d::from([0., 1.]),
            Direction::Stay => Vec2d::from([0f64; 2])
        }
    }
}

impl From<Key> for Direction {
    fn from(key: Key) -> Self {
        match key {
            Key::Up => Direction::Up,
            Key::Down => Direction::Down,
            Key::Right => Direction::Right,
            Key::Left => Direction::Left,
            _ => Direction::Stay
        }
    }
}

pub const RED: [f32; 4] = [1., 0., 0., 1.];
pub const GREEN: [f32; 4] = [0., 1., 0., 1.];
pub const BLUE: [f32; 4] = [0., 0., 1., 1.];
pub const WHITE: [f32; 4] = [1., 1., 1., 1.];
pub const BLACK: [f32; 4] = [0., 0., 0., 1.];
