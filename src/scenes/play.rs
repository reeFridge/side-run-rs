use std::net::TcpStream;
use std::collections::HashMap;
use connection::{Connection, NetToken, EventType};
use piston_window::types::Color;
use piston_window::*;
use scenes::common::*;
use scenes::scene::{Scene, SceneInstance, BaseSwitcher, Switcher};
use scenes::menu::Menu;

const W_HEIGHT: f32 = 1000.0;
const W_WIDTH: f32 = 1000.0;

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
    color: Color,
    velocity: [f32; 2],
    direction: [f32; 2]
}

impl GameObject {
    fn new(x: f32, y: f32, color: Color) -> GameObject {
        GameObject { pos: Point::<f32>::new(x, y), color: color, velocity: [0f32; 2], direction: [0., -1.] }
    }

    fn move_to(&mut self, direction: Direction, units: f32) -> Option<Point<f32>> {
        let x = self.pos.x().clone();
        let y = self.pos.y().clone();

        let intersect = (y - units < 0.0) as u8 |
            (((y + units > W_HEIGHT) as u8) << 1) |
            (((x - units < 0.0) as u8) << 2) |
            (((x + units > W_WIDTH) as u8) << 3);

        let intersect = (intersect >> direction.clone() as u8) & 1 == 1;

        if !intersect {
            let dir: [f32; 2] = direction.into();
            self.velocity = [dir[0] * units, dir[1] * units];

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

#[derive(Clone)]
pub struct PlayerConfig {
    pub name: String,
    pub color: Color
}

// if connection is not established player will be at   players[0]
// else controllable player will be at                  players[connection.token]
pub struct Play {
    switcher: BaseSwitcher,
    free_area: Rect<f32>,
    viewport: ViewPort,
    objects: Vec<GameObject>,
    players: HashMap<NetToken, Player>,
    connection: Option<Connection>,
    player_config: PlayerConfig,
    cursor: [f64; 2]
}

impl Play {
    pub fn new(auto_connect: Option<String>, player_config: PlayerConfig) -> Play {
        let objects = vec![
            GameObject::new(200.0, 300.0, WHITE),
            GameObject::new(500.0, 100.0, WHITE),
            GameObject::new(50.0, 40.0, WHITE)
        ];

        let mut play = Play {
            switcher: BaseSwitcher::new(None),
            objects: objects,
            viewport: ViewPort::new(0.0, 0.0),
            players: HashMap::new(),
            free_area: Rect::<f32>::new(200., 150., 600., 450.),
            connection: None,
            player_config: player_config,
            cursor: [0f64; 2]
        };

        if let Some(addr) = auto_connect {
            match play.connect(addr) {
                Err(err) => println!("Failed to connect: {}", err),
                _ => ()
            }
        }

        play
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

    fn spawn_player(&mut self, token: NetToken, pos: Point<f32>, name: String, color: Color) {
        let idx = self.objects.len();
        self.objects.push(GameObject::new(pos.x().clone(), pos.y().clone(), color));

        self.players.insert(token, Player {
            name: name,
            obj_index: idx
        });
    }

    fn spawn_self_player(&mut self, pos: Point<f32>) {
        let token = match self.connection {
            Some(Connection { ref token, .. }) => token.clone(),
            None => 0 as NetToken
        };

        let PlayerConfig { name, color } = self.player_config.clone();

        self.spawn_player(token, pos.clone(), name.clone(), color.clone());

        self.connection.as_mut()
            .and_then(|ref mut connection| Some(connection.send_spawn_event(name, pos, color)));
    }

    fn update_player_pos(&mut self, token: NetToken, new_pos: Point<f32>) {
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

impl Scene for Play {
    fn switcher(&mut self) -> &mut Switcher {
        &mut self.switcher
    }

    fn update(&mut self, dt: f64) -> GameResult<()> {
        for obj in self.objects.iter_mut() {
            if obj.velocity != [0f32, 0f32] {
                obj.pos.vec[0] += (obj.velocity[0] * 40.) * dt as f32;
                obj.pos.vec[1] += (obj.velocity[1] * 40.) * dt as f32;
                obj.velocity[0] -= obj.velocity[0] * 0.1;
                obj.velocity[1] -= obj.velocity[1] * 0.1;
            }
        }

        self.connection.as_mut()
            .and_then(|ref mut connection| {
                connection.listen_events()
            })
            .and_then(|(event_type, data)| {
                match event_type {
                    EventType::Spawn => {
                        let (token, name, pos, color) = Connection::parse_spawn_event(data).unwrap();
                        self.spawn_player(token, pos, name, color);
                    },
                    EventType::UpdatePos => {
                        let (token, pos) = Connection::parse_update_pos_event(data).unwrap();
                        self.update_player_pos(token, pos);
                    }
                };

                Some(())
            });

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, graphics: &mut G2d) -> GameResult<()> {
        clear(BLACK, graphics);

        let cursor = self.cursor.clone();

        self.player()
            .and_then(|player| {
                Some((player.pos.clone(), player.direction.clone()))
            })
            .and_then(|(global_pos, dir)| {
                let screen_pos = self.viewport.convert_world_pos(global_pos);
                let pos = ctx.transform.trans(screen_pos.x().clone() as f64, screen_pos.y().clone() as f64);
                let line = [
                    0.,
                    0.,
                    (dir[0] * 20.) as f64,
                    (dir[1] * 20.) as f64
                ];
                Line::new(WHITE, 0.5)
                    .draw_arrow(line, 10., &ctx.draw_state, pos, graphics);

                Some(())
            });

        self.player()
            .and_then(|player| {
                Some(player.pos.clone())
            })
            .and_then(|global_pos| {
                let screen_pos = self.viewport.convert_world_pos(global_pos);
                let line = [screen_pos.x().clone() as f64, screen_pos.y().clone() as f64, cursor[0], cursor[1]];
                Line::new(WHITE, 0.5)
                    .draw_arrow(line, 10., &ctx.draw_state, ctx.transform.clone(), graphics);

                Some(())
            });

        for obj in self.objects.iter() {
            let screen_pos = self.viewport.convert_world_pos(obj.pos.clone());
            let pos = ctx.transform.trans(screen_pos.x().clone() as f64, screen_pos.y().clone() as f64);
            let square = rectangle::centered_square(0., 0., 10.);
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

    fn key_press(&mut self, button: Button) {
        //TODO: Fix handling space key only if player not spawned
        if let Button::Keyboard(key) = button {
            let direction = Direction::from(key);
            let step = 10f32;

            self.player()
                .and_then(|ref mut player| {
                    player.move_to(direction.clone(), step)
                })
                .and_then(|new_pos| {
                    if let Some(ref mut connection) = self.connection {
                        connection.send_update_pos_event(new_pos.clone()).unwrap();
                    }

                    let local = self.viewport.convert_world_pos(new_pos);
                    let x = local.x().clone();
                    let y = local.y().clone();

                    let free_area = &self.free_area;

                    let intersect = (y < free_area.top_left.y().clone()) as u8 |
                        (((y > free_area.bottom_right.y().clone()) as u8) << 1) |
                        (((x < free_area.top_left.x().clone()) as u8) << 2) |
                        (((x > free_area.bottom_right.x().clone()) as u8) << 3);

                    if (intersect >> direction.clone() as u8) & 1 == 1 {
                        self.viewport.move_to(direction, step);

                        Some(())
                    } else {
                        None
                    }
                })
                .or_else(|| {
                    match key {
                        Key::Space => {
                            let spawn_pos = Point::<f32>::new(400., 300.);
                            self.spawn_self_player(spawn_pos.clone());
                        },
                        _ => ()
                    };

                    None
                });
        }
    }

    fn mouse_move(&mut self, cursor: [f64; 2]) {
        self.cursor = cursor;
    }
}
