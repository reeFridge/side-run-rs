use std::net::TcpStream;
use std::collections::HashMap;
use connection::{Connection, NetToken, EventType};
use piston_window::types::Color;
use piston_window::*;
use scenes::common::*;
use scenes::scene::{Scene, SceneInstance, BaseSwitcher, Switcher};
use scenes::menu::Menu;
use std::f64;
use vecmath::*;
use piston_window::math::*;
use piston_window::types::Rectangle as Rect;

const W_HEIGHT: f64 = 1000.0;
const W_WIDTH: f64 = 1000.0;

struct Camera {
    obj: GameObject
}

impl Camera {
    fn new(x: f64, y: f64) -> Camera {
        Camera { obj: GameObject::new(x, y, BLUE) }
    }

    fn world_to_screen(&self, world: Vec2d) -> Vec2d {
        sub(world,self.get_pos())
    }
    fn screen_to_world(&self, screen: Vec2d) -> Vec2d {
        add(self.get_pos(), screen)
    }

    fn get_pos(&self) -> Vec2d {
        self.obj.get_pos()
    }

    fn move_to(&mut self, direction: Vec2d, speed: f64) {
        self.obj.move_to(direction, speed);
    }
}

trait Position {
    fn x_y(&self) -> (f64, f64);
}

impl Position for Vec2d {
    fn x_y(&self) -> (f64, f64) {
        (self[0], self[1])
    }
}

struct GameObject {
    pos: Vec2d,
    rotation: f64,
    color: Color,
    velocity: Vec2d
}

impl GameObject {
    fn new(x: f64, y: f64, color: Color) -> GameObject {
        GameObject {
            rotation: 0.,
            pos: Vec2d::from([x, y]),
            color: color,
            velocity: Vec2d::from([0., 0.])
        }
    }

    fn get_pos(&self) -> Vec2d {
        self.pos.clone()
    }

    fn update_velocity(&mut self, dt: f64) {
        if vec2_len(self.velocity) != 0. {
            let transform = translate(mul_scalar(self.velocity, dt));
            let resistance = 0.9;
            self.pos = transform_pos(transform, self.pos);
            self.velocity = mul_scalar(self.velocity, resistance);
        } else {
            self.velocity = Vec2d::from([0., 0.]);
        }
    }

    fn look_at(&mut self, target: Vec2d) -> Option<f64> {
        let eye = self.get_pos();
        let current = sub(
            sub(eye, Vec2d::from([0., 20.])),
            eye
        );
        let target = sub(Vec2d::from(target), eye);
        let n_target = vec2_normalized(target);
        let n_current = vec2_normalized(current);
        let mut angle = vec2_dot(n_current, n_target).acos();

        if n_target[0] < 0. {
            angle = f64::consts::PI * 2. - angle;
        }

        self.rotation = angle;

        Some(angle)
    }

    fn move_to(&mut self, direction: Vec2d, speed: f64) -> Option<Vec2d> {
        self.velocity = mul_scalar(direction, speed);

        Some(self.get_pos())
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
    free_area: Rect,
    camera: Camera,
    objects: Vec<GameObject>,
    players: HashMap<NetToken, Player>,
    connection: Option<Connection>,
    player_config: PlayerConfig,
    cursor: [f64; 2]
}

impl Play {
    pub fn new(auto_connect: Option<String>, player_config: PlayerConfig) -> Play {
        let objects = vec![
            GameObject::new(200.0, 300.0, GREEN),
            GameObject::new(500.0, 100.0, BLUE),
            GameObject::new(50.0, 40.0, WHITE)
        ];

        let mut play = Play {
            switcher: BaseSwitcher::new(None),
            objects: objects,
            camera: Camera::new(0.0, 0.0),
            players: HashMap::new(),
            free_area: Rect::from([200., 150., 600., 450.]),
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

    fn spawn_player(&mut self, token: NetToken, pos: Vec2d, name: String, color: Color) {
        let idx = self.objects.len();
        self.objects.push(GameObject::new(pos[0], pos[1], color));

        self.players.insert(token, Player {
            name: name,
            obj_index: idx
        });
    }

    fn spawn_self_player(&mut self, pos: Vec2d) {
        let token = match self.connection {
            Some(Connection { ref token, .. }) => token.clone(),
            None => 0 as NetToken
        };

        let PlayerConfig { name, color } = self.player_config.clone();

        self.spawn_player(token, pos.clone(), name.clone(), color.clone());

        self.connection.as_mut()
            .and_then(|ref mut connection| Some(connection.send_spawn_event(name, pos, color)));
    }

    fn update_player_pos(&mut self, token: NetToken, new_pos: Vec2d) {
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
            obj.update_velocity(dt);
        }

        self.camera.obj.update_velocity(dt);

        let cursor = self.camera.screen_to_world(self.cursor);

        let rot = self.player()
            .and_then(|player_obj| {
                player_obj.look_at(cursor)
            });

        /*self.connection.as_mut()
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
            });*/

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, graphics: &mut G2d) -> GameResult<()> {
        clear(BLACK, graphics);

        /*self.player()
            .and_then(|player| {
                Some((player.pos.clone(), player.rotation.clone()))
            })
            .and_then(|(global_pos, rotation)| {
                let screen_pos = self.camera.world_to_screen(global_pos);
                let pos = ctx.transform
                    .trans(screen_pos[0], screen_pos[1])
                    .rot_rad(rotation);

                let (dir_up, dir_down): ([f64; 2], [f64; 2]) = (Direction::Up.into(), Direction::Down.into());

                let line_up = [0., 0., 0., 20. * dir_up[1]];
                let line_down = [0., 0., 0., 20. * dir_down[1]];

                Line::new(GREEN, 0.5)
                    .draw_arrow(line_up, 5., &ctx.draw_state, pos, graphics);

                Line::new(RED, 0.5)
                    .draw_arrow(line_down, 5., &ctx.draw_state, pos, graphics);

                Some(())
            });*/
        self.player()
            .and_then(|player| {
                Some(player.get_pos())
            })
            .and_then(|player_pos| {
                let screen_pos = self.camera.world_to_screen(player_pos);
                let (x, y) = screen_pos.x_y();
                let free_camera_area = &self.free_area;

                let intersect = (y - 10. < free_camera_area[1]) as u8 |
                    (((y + 10. > free_camera_area[3]) as u8) << 1) |
                    (((x - 10. < free_camera_area[0]) as u8) << 2) |
                    (((x + 10. > free_camera_area[2]) as u8) << 3);

                let mut dir = Vec2d::from([0f64; 2]);

                fn add_direction_if_intersect(direction: Direction, to: Vec2d, intersect: u8) -> Vec2d {
                    if (intersect >> direction.clone() as u8) & 1 == 1 {
                        add(to, direction.into())
                    } else {
                        add(to, [0f64; 2])
                    }
                }

                dir = add_direction_if_intersect(Direction::Up, dir, intersect);
                dir = add_direction_if_intersect(Direction::Down, dir, intersect);
                dir = add_direction_if_intersect(Direction::Right, dir, intersect);
                dir = add_direction_if_intersect(Direction::Left, dir, intersect);

                if dir != [0f64; 2] {
                    let camera_pos = self.camera.get_pos();
                    let c_x = (free_camera_area[0] - free_camera_area[2]) / 2.;
                    let c_y = (free_camera_area[1] - free_camera_area[3]) / 2.;
                    let center = sub([free_camera_area[0], free_camera_area[1]], [c_x, c_y]);
                    self.camera.move_to(vec2_normalized(dir), 150.);
                }

                Some(())
            });

        for obj in self.objects.iter() {
            let screen_pos = self.camera.world_to_screen(obj.get_pos());
            let pos = multiply(ctx.transform, translate(screen_pos)).rot_rad(obj.rotation);
            let square = rectangle::centered_square(0., 0., 10.);
            rectangle(obj.color.clone(), square, pos, graphics);
        }

        self.player()
            .and_then(|player_obj| {
                Some((player_obj.get_pos(), player_obj.rotation))
            })
            .and_then(|(pos, rot)| {
                let screen_pos = self.camera.world_to_screen(pos);
                let player_transform = multiply(ctx.transform, translate(screen_pos)).rot_rad(rot);
                let right = player_transform.rot_rad(f64::consts::PI / 4.);
                let left = player_transform.rot_rad(-f64::consts::PI / 4.);

                line(RED, 0.5, [0.,-20.,0.,-40.], right, graphics);
                line(WHITE, 0.5, [0.,-20.,0.,-40.], player_transform, graphics);
                line(GREEN, 0.5, [0.,-20.,0.,-40.], left, graphics);

                Some(())
            });

        let area = &self.free_area;
        let rect = rectangle::rectangle_by_corners(area[0], area[1], area[2], area[3]);
        let area_border = Rectangle::new_border([0., 0., 1., 0.1], 0.5);
        area_border.draw(rect, &ctx.draw_state, ctx.transform.clone(), graphics);

        Ok(())
    }

    fn key_press(&mut self, button: Button) {
        //TODO: Fix handling space key only if player not spawned
        if let Button::Keyboard(key) = button {
            self.player()
                .and_then(|ref mut player| {
                    player.move_to(Vec2d::from(Direction::from(key)), 200.)
                })
                .or_else(|| {
                    match key {
                        Key::Space => {
                            let spawn_pos = Vec2d::from([400., 300.]);
                            self.spawn_self_player(spawn_pos);
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
