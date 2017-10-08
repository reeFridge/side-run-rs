//use ggez::event::{EventHandler, Keycode, Mod};
//use ggez::{Context};
//use ggez::graphics;
//use ggez::graphics::{Point, DrawMode, Rect, Color};
//use std::time::Duration;
//use std::net::TcpStream;
//use std::io::{Read};
use std::collections::HashMap;
//use connection::{Connection, NetToken, EventType};
type GameResult<T> = Result<T, String>;
type NetToken = usize;

use piston_window::types::Color;
use piston_window::*;

struct Point<T> {
    vec: [T; 2]
}

impl<T: Clone> Point<T> {
    fn new(x: T, y: T) -> Self {
        Point { vec: [x, y] }
    }

    fn x(&self) -> &T {
        &self.vec[0]
    }

    fn y(&self) -> &T {
        &self.vec[1]
    }

    fn set(&mut self, point: Point<T>) {
        self.vec = point.vec;
    }

    fn set_x(&mut self, x: T) {
        self.vec[0] = x;
    }

    fn set_y(&mut self, y: T) {
        self.vec[1] = y;
    }

    fn clone(&self) -> Point<T> {
        Point { vec: [self.x().clone(), self.y().clone()] }
    }
}

trait Movable {
    fn translate_by_direction(&mut self, direction: Direction, units: f32);
}

impl Movable for Point<f32> {
    fn translate_by_direction(&mut self, direction: Direction, units: f32) {
        match direction {
            Direction::Up => self.vec[1] -= units,
            Direction::Down => self.vec[1] += units,
            Direction::Left => self.vec[0] -= units,
            Direction::Right => self.vec[0] += units,
        };
    }
}

#[derive(Clone)]
enum Direction {
    Up,
    Down,
    Left,
    Right
}

struct Rect<T: Clone> {
    top_left: Point<T>,
    bottom_right: Point<T>
}

impl<T: Clone> Rect<T> {
    fn new(x: T, y: T, w: T, h: T) -> Self {
        Rect {
            top_left: Point::<T>::new(x, y),
            bottom_right: Point::<T>::new(w, h)
        }
    }
}

const W_HEIGHT: f32 = 1000.0;
const W_WIDTH: f32 = 1000.0;
const RED: [f32; 4] = [1., 0., 0., 1.];
const GREEN: [f32; 4] = [0., 1., 0., 1.];
const BLUE: [f32; 4] = [0., 0., 1., 1.];
const WHITE: [f32; 4] = [1., 1., 1., 1.];
const BLACK: [f32; 4] = [0., 0., 0., 1.];

struct ViewPort {
    pos: Point<f32>
}

impl ViewPort {
    fn new(x: f32, y: f32) -> ViewPort {
        ViewPort { pos: Point::<f32>::new(x, y) }
    }

    fn convert_world_pos(&self, world_pos: Point<f32>) -> Point<f32> {
        Point::<f32>::new(world_pos.x() - self.pos.x(), world_pos.y() - self.pos.y())
    }

    fn move_to(&mut self, direction: Direction, units: f32) {
        self.pos.translate_by_direction(direction, units);
    }
}

struct GameObject {
    pos: Point<f32>,
    color: Color
}

impl GameObject {
    fn new(x: f32, y: f32, color: Color) -> GameObject {
        GameObject { pos: Point::<f32>::new(x, y), color: color }
    }

    fn move_to(&mut self, direction: Direction, units: f32) -> Option<Point<f32>> {
        let x = self.pos.x().clone();
        let y = self.pos.y().clone();

        let intersect = (y - units < 0.0) as u8 |
            (((y + units > W_HEIGHT - 20.) as u8) << 1) |
            (((x - units < 0.0) as u8) << 2) |
            (((x + units > W_WIDTH - 20.) as u8) << 3);

        let intersect = (intersect >> direction.clone() as u8) & 1 == 1;

        if !intersect {
            self.pos.translate_by_direction(direction, units);

            Some(self.pos.clone())
        } else {
            None
        }
    }
}

struct Player {
    name: String,
    obj_index: usize
}

// if connection is not established player will be at   players[0]
// else controllable player will be at                  players[connection.token]
pub struct State {
    free_area: Rect<f32>,
    viewport: ViewPort,
    objects: Vec<GameObject>,
    players: HashMap<NetToken, Player>
    //connection: Option<Connection>
}

impl State {
    pub fn new() -> GameResult<State> {
        let objects = vec![
            GameObject::new(200.0, 300.0, WHITE),
            GameObject::new(500.0, 100.0, WHITE),
            GameObject::new(50.0, 40.0, WHITE)
        ];

        let s = State {
            objects: objects,
            viewport: ViewPort::new(0.0, 0.0),
            players: HashMap::new(),
            free_area: Rect::<f32>::new(200., 150., 600., 450.)
            //connection: None
        };

        Ok(s)
    }

    pub fn update(&mut self, dt: f64) -> GameResult<()> {
        match self.player() {
            Some(&mut GameObject { pos: ref player_pos, .. }) => Some(player_pos.clone()),
            None => None
        }.and_then(|world| {
            let units = 10.;
            let local = self.viewport.convert_world_pos(world.clone());
            let x = local.x().clone();
            let y = local.y().clone();

            let free_area = &self.free_area;

            let intersect = (y < free_area.top_left.y().clone()) as u8 |
                (((y > free_area.bottom_right.y().clone() - 20.) as u8) << 1) |
                (((x < free_area.top_left.x().clone()) as u8) << 2) |
                (((x > free_area.bottom_right.x() - 20.) as u8) << 3);

            if (intersect >> Direction::Up as u8) & 1 == 1 {
                Some((Direction::Up, units))
            } else if (intersect >> Direction::Down as u8) & 1 == 1 {
                Some((Direction::Down, units))
            } else if (intersect >> Direction::Left as u8) & 1 == 1 {
                Some((Direction::Left, units))
            } else if (intersect >> Direction::Right as u8) & 1 == 1 {
                Some((Direction::Right, units))
            } else {
                None
            }
        }).and_then(|(direction, units)| {
            self.viewport.move_to(direction, units);

            Some(())
        });
        /*

                let mut buf = [0u8; 64];
                if let Some(Connection { ref mut socket, .. }) = self.connection {
                    socket.set_read_timeout(Some(Duration::from_millis(10))).unwrap();

                    match socket.read(&mut buf) {
                        Ok(_) => {
                            let (event, raw_data) = buf.split_at(5);

                            Some((Connection::parse_event_type(&event), raw_data))
                        },
                        Err(_) => None
                    }
                } else { None }.and_then(|(event_type, raw_data)| {
                    match event_type {
                        Some(EventType::Spawn) => {
                            let (token, name, pos, color) = Connection::parse_spawn_event(&raw_data).unwrap();
                            self.spawn_player(token, name, pos, color);
                        },
                        Some(EventType::UpdatePos) => {
                            let (token, pos) = Connection::parse_update_pos_event(&raw_data).unwrap();
                            self.update_player_pos(token, pos);
                        },
                        None => ()
                    };

                    Some(())
                });
        */

        Ok(())
    }

    /*    pub fn connect(&mut self, host: String) -> Result<(), String> {
            match TcpStream::connect(host) {
                Ok(stream) => match Connection::new(stream) {
                    Ok(connection) => {
                        println!("connection established, net_token= {}", connection.token);
                        self.connection = Some(connection);

                        Ok(())
                    },
                    Err(err) => Err(err)
                },
                Err(e) => Err(format!("{:?}", e.kind()))
            }
        }*/

    fn spawn_player(&mut self, token: NetToken, name: String, pos: Point<f32>, color: Color) {
        let idx = self.objects.len();
        self.objects.push(GameObject::new(pos.x().clone(), pos.y().clone(), color));

        self.players.insert(token, Player {
            name: name,
            obj_index: idx
        });
    }
    /*
        fn update_player_pos(&mut self, token: NetToken, new_pos: Point) {
            match self.players.get_mut(&token) {
                Some(&mut Player { obj_index: ref idx, .. }) => self.objects.get_mut(idx.clone()),
                None => None
            }.and_then(|obj| {
                obj.pos = new_pos;

                Some(())
            });
        }*/

    fn player(&mut self) -> Option<&mut GameObject> {
        //        let token = match self.connection {
        //            Some(Connection { ref token, .. }) => token.clone(),
        //            None => 0 as NetToken
        //        };
        let token = 0 as NetToken;

        match self.players.get_mut(&token) {
            Some(&mut Player { obj_index: ref idx, .. }) => self.objects.get_mut(idx.clone()),
            None => None
        }
    }

    pub fn draw(&mut self, ctx: &mut Context, graphics: &mut G2d) -> GameResult<()> {
        clear(BLACK, graphics);

        for obj in self.objects.iter() {
            let screen_pos = self.viewport.convert_world_pos(obj.pos.clone());
            let pos = ctx.transform.trans(screen_pos.x().clone() as f64, screen_pos.y().clone() as f64);
            let square = rectangle::square(0., 0., 20.);
            rectangle(obj.color.clone(), square, pos, graphics);
        }

        //Side-scroll area
        let square = rectangle::rectangle_by_corners(
            self.free_area.top_left.x().clone() as f64,
            self.free_area.top_left.y().clone() as f64,
            self.free_area.bottom_right.x().clone() as f64,
            self.free_area.bottom_right.y().clone() as f64
        );
        let area_border = Rectangle::new_border([0., 0., 1., 0.1], 0.5);
        area_border.draw(square, &ctx.draw_state, ctx.transform.clone(), graphics);

        let world = vec![
            self.viewport.convert_world_pos(Point::<f32>::new(0., 0.)),
            self.viewport.convert_world_pos(Point::<f32>::new(W_WIDTH, W_HEIGHT))
        ];
        let view_pos = self.viewport.pos.clone();

        //World area
        let pos = ctx.transform.trans(world[0].x().clone() as f64, world[0].y().clone() as f64);
        let square = rectangle::rectangle_by_corners(
            0.,
            0.,
            (world[1].x() + view_pos.x()) as f64,
            (world[1].y() + view_pos.y()) as f64
        );
        let area_border = Rectangle::new_border(WHITE, 0.5);
        area_border.draw(square, &ctx.draw_state, pos, graphics);
        Ok(())
    }

    pub fn key_press(&mut self, button: Button) {
        if let Button::Keyboard(key) = button {
            match self.player() {
                Some(ref mut player) => {
                    match key {
                        Key::Up => player.move_to(Direction::Up, 10.0),
                        Key::Down => player.move_to(Direction::Down, 10.0),
                        Key::Left => player.move_to(Direction::Left, 10.0),
                        Key::Right => player.move_to(Direction::Right, 10.0),
                        _ => None
                    }
                }
                None => None
            }.and_then(|new_pos| {
                //            if let Some(ref mut connection) = self.connection {
                //                connection.send_update_pos_event(new_pos).unwrap();
                //            }

                Some(())
            }).or_else(|| {
                match key {
                    Key::Space => {
                        //                    let token = match self.connection {
                        //                        Some(Connection { ref token, .. }) => token.clone(),
                        //                        None => 0 as NetToken
                        //                    };
                        /*

                                                let name = "Fratyz".to_string();
                                                let start_pos = Point::new(self.viewport.w as f32 / 2.0, self.viewport.h as f32 / 2.0);
                                                let color = Color::from((255, 0, 255));

                                                let name = "Reef".to_string();
                                                let start_pos = Point::new(self.viewport.w as f32 / 2.0, self.viewport.h as f32 / 2.0);
                                                let color = Color::from((0, 255, 0));
                        */

                        let token = 0 as NetToken;
                        let name = "Fridge".to_string();
                        let start_pos = Point::<f32>::new(400., 300.);

                        self.spawn_player(token, name, start_pos, RED);

                        //                    if let Some(ref mut connection) = self.connection {
                        //                        connection.send_spawn_event(name, start_pos, color).unwrap();
                        //                    }
                    }
                    _ => ()
                };

                None
            });
        }
    }
}
