use ggez::event::{EventHandler, Keycode, Mod};
use ggez::{GameResult, Context};
use ggez::graphics;
use ggez::graphics::{Point, DrawMode, Rect, Color};
use std::time::Duration;
use std::net::TcpStream;
use std::io::{Read};
use std::collections::HashMap;
use connection::{Connection, NetToken, EventType};

enum Direction {
    Up,
    Down,
    Left,
    Right
}

const W_HEIGHT: f32 = 1000.0;
const W_WIDTH: f32 = 1000.0;

struct ViewPort {
    pos: Point,
    w: u32,
    h: u32,
}

impl ViewPort {
    fn new(x: f32, y: f32, w: u32, h: u32) -> ViewPort {
        ViewPort { pos: Point { x: x, y: y }, w: w, h: h }
    }

    fn convert_world_pos(&self, world_pos: Point) -> Point {
        Point { x: world_pos.x - self.pos.x, y: world_pos.y - self.pos.y }
    }

    fn move_to(&mut self, direction: Direction, units: f32) {
        match direction {
            Direction::Up => self.pos.y -= units,
            Direction::Down => self.pos.y += units,
            Direction::Left => self.pos.x -= units,
            Direction::Right => self.pos.x += units,
        };
    }
}

struct GameObject {
    pos: Point,
    color: Color
}

impl GameObject {
    fn new(x: f32, y: f32, color: Color) -> GameObject {
        GameObject { pos: Point { x: x, y: y}, color: color }
    }

    fn move_to(&mut self, direction: Direction, units: f32) -> Option<Point> {
        match direction {
            Direction::Up => {
                if self.pos.y - units > 0.0 {
                    self.pos.y -= units;

                    Some(self.pos.clone())
                } else { None }
            },
            Direction::Down => {
                if self.pos.y + units < W_HEIGHT {
                    self.pos.y += units;

                    Some(self.pos.clone())
                } else { None }
            },
            Direction::Left => {
                if self.pos.x - units > 0.0 {
                    self.pos.x -= units;

                    Some(self.pos.clone())
                } else { None }
            },
            Direction::Right => {
                if self.pos.x + units < W_WIDTH {
                    self.pos.x += units;

                    Some(self.pos.clone())
                } else { None }
            }
        }
    }
}

// if connection is not established player will be at   players[0]
// else controllable player will be at                  players[connection.token]
pub struct State {
    free_area: Rect,
    viewport: ViewPort,
    objects: Vec<GameObject>,
    players: HashMap<NetToken, Player>,
    connection: Option<Connection>
}

struct Player {
    name: String,
    obj_index: usize
}

impl State {
    pub fn new() -> GameResult<State> {
        let objects = vec![
            GameObject::new(200.0, 300.0, Color::from((255, 255, 255))),
            GameObject::new(500.0, 100.0, Color::from((255, 255, 255))),
            GameObject::new(50.0, 40.0, Color::from((255, 255, 255)))
        ];

        let s = State {
            objects: objects,
            viewport: ViewPort::new(0.0, 0.0, 800, 600),
            players: HashMap::new(),
            free_area: Rect {
                x: 200.0,
                y: 150.0,
                w: 400.0,
                h: 300.0,
            },
            connection: None
        };

        Ok(s)
    }

    pub fn connect(&mut self, host: String) -> Result<(), String> {
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
    }

    fn spawn_player(&mut self, token: NetToken, name: String, pos: Point, color: Color) {
        let idx = self.objects.len();
        self.objects.push(GameObject::new(pos.x, pos.y, color.clone()));

        self.players.insert(token.clone(), Player {
            name: name.clone(),
            obj_index: idx
        });
    }

    fn update_player_pos(&mut self, token: NetToken, new_pos: Point) {
        match self.players.get_mut(&token) {
            Some(&mut Player { obj_index: ref idx, .. }) => self.objects.get_mut(idx.clone()),
            None => None
        }.and_then(|obj| {
            obj.pos = new_pos;

            Some(())
        });
    }

    fn player(&mut self) -> Option<&mut GameObject> {
        let token = match self.connection {
            Some(Connection { ref token, .. }) => token.clone(),
            None => 0 as NetToken
        };

        match self.players.get_mut(&token) {
            Some(&mut Player { obj_index: ref idx, .. }) => self.objects.get_mut(idx.clone()),
            None => None
        }
    }
}

impl EventHandler for State {
    fn update(&mut self, _ctx: &mut Context, _dt: Duration) -> GameResult<()> {
        match self.player() {
            Some(&mut GameObject { pos: ref player_pos, .. }) => Some(player_pos.clone()),
            None => None
        }.and_then(|world| {
            let local = self.viewport.convert_world_pos(world.clone());
            let free_area = &self.free_area;

            let delta = Point {
                x: free_area.x + (free_area.w - local.x),
                y: free_area.y + (free_area.h - local.y)
            };

            if world.x > 0.0 && world.y > 0.0 {
                if delta.x < 0.0 {
                    Some((Direction::Right, -delta.x))
                } else if delta.x > free_area.w {
                    Some((Direction::Left, free_area.x - local.x))
                } else if delta.y < 0.0 {
                    Some((Direction::Down, -delta.y))
                } else if delta.y > free_area.h {
                    Some((Direction::Up, free_area.y - local.y))
                } else {
                    None
                }
            } else {
                None
            }
        }).and_then(|(direction, units)| {
            self.viewport.move_to(direction, units);

            Some(())
        });

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

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::set_background_color(ctx, Color::from((0, 0, 0)));
        graphics::clear(ctx);

        for obj in self.objects.iter() {
            graphics::set_color(ctx, obj.color.clone())?;
            let screen_pos = self.viewport.convert_world_pos(obj.pos.clone());
            graphics::rectangle(ctx, DrawMode::Fill, Rect::new(
                screen_pos.x,
                screen_pos.y,
                20.0,
                20.0
            ))?;
        }

        //Side-scroll area
        graphics::set_color(ctx, Color::from((0, 0, 255)))?;
        graphics::rectangle(ctx, DrawMode::Line, Rect::new(
            self.free_area.w,
            self.free_area.h,
            self.free_area.w,
            self.free_area.h
        ))?;

        let world_edges = vec![
            self.viewport.convert_world_pos(Point::new(0.0, 0.0)),
            self.viewport.convert_world_pos(Point::new(W_WIDTH, 0.0)),
            self.viewport.convert_world_pos(Point::new(W_WIDTH, W_HEIGHT)),
            self.viewport.convert_world_pos(Point::new(0.0, W_HEIGHT))
        ];

        //World area
        graphics::set_color(ctx, Color::from((255, 255, 255)))?;
        graphics::polygon(ctx, DrawMode::Line, &world_edges)?;

        graphics::present(ctx);
        Ok(())
    }

    fn key_down_event(&mut self, _keycode: Keycode, _: Mod, _repeat: bool) {
        match self.player() {
            Some(ref mut player) => {
                match _keycode {
                    Keycode::Up => player.move_to(Direction::Up, 10.0),
                    Keycode::Down => player.move_to(Direction::Down, 10.0),
                    Keycode::Left => player.move_to(Direction::Left, 10.0),
                    Keycode::Right => player.move_to(Direction::Right, 10.0),
                    _ => None
                }
            },
            None => None
        }.and_then(|new_pos| {
            if let Some(ref mut connection) = self.connection {
                connection.send_update_pos_event(new_pos).unwrap();
            }

            Some(())
        }).or_else(|| {
            match _keycode {
                Keycode::Space => {
                    let token = match self.connection {
                        Some(Connection { ref token, .. }) => token.clone(),
                        None => 0 as NetToken
                    };

                    //                    let name = "Fratyz".to_string();
                    //                    let start_pos = Point::new(self.viewport.w as f32 / 2.0, self.viewport.h as f32 / 2.0);
                    //                    let color = Color::from((255, 0, 255));

                    //                    let name = "Reef".to_string();
                    //                    let start_pos = Point::new(self.viewport.w as f32 / 2.0, self.viewport.h as f32 / 2.0);
                    //                    let color = Color::from((0, 255, 0));

                    let name = "Fridge".to_string();
                    let start_pos = Point::new(self.viewport.w as f32 / 2.0, self.viewport.h as f32 / 2.0);
                    let color = Color::from((255, 0, 0));

                    self.spawn_player(token, name.clone(), start_pos.clone(), color.clone());

                    if let Some(ref mut connection) = self.connection {
                        connection.send_spawn_event(name, start_pos, color).unwrap();
                    }
                },
                _ => ()
            };

            None
        });
    }
}