use piston_window::*;

pub type GameResult<T> = Result<T, String>;

pub struct Point<T> {
    vec: [T; 2]
}

impl<T: Clone> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Point { vec: [x, y] }
    }

    pub fn x(&self) -> &T {
        &self.vec[0]
    }

    pub fn y(&self) -> &T {
        &self.vec[1]
    }

    pub fn set(&mut self, point: Point<T>) {
        self.vec = point.vec;
    }

    pub fn set_x(&mut self, x: T) {
        self.vec[0] = x;
    }

    pub fn set_y(&mut self, y: T) {
        self.vec[1] = y;
    }

    pub fn clone(&self) -> Point<T> {
        Point { vec: [self.x().clone(), self.y().clone()] }
    }
}

pub trait Movable {
    fn translate_by_direction(&mut self, direction: Direction, units: f32);
}

impl Movable for Point<f32> {
    fn translate_by_direction(&mut self, direction: Direction, units: f32) {
        match direction {
            Direction::Up => self.vec[1] -= units,
            Direction::Down => self.vec[1] += units,
            Direction::Left => self.vec[0] -= units,
            Direction::Right => self.vec[0] += units,
            Direction::Stay => ()
        };
    }
}

#[derive(Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    Stay
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

pub struct Rect<T: Clone> {
    pub top_left: Point<T>,
    pub bottom_right: Point<T>
}

impl<T: Clone> Rect<T> {
    pub fn new(x: T, y: T, w: T, h: T) -> Self {
        Rect {
            top_left: Point::<T>::new(x, y),
            bottom_right: Point::<T>::new(w, h)
        }
    }
}

pub const RED: [f32; 4] = [1., 0., 0., 1.];
pub const GREEN: [f32; 4] = [0., 1., 0., 1.];
pub const BLUE: [f32; 4] = [0., 0., 1., 1.];
pub const WHITE: [f32; 4] = [1., 1., 1., 1.];
pub const BLACK: [f32; 4] = [0., 0., 0., 1.];
