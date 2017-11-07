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
use cgmath;
use collision as cgcoll;
use collision::ContinuousTransformed;
use collision::HasAabb;
use cgmath::MetricSpace;
use std::cmp::Ordering;

const W_HEIGHT: f64 = 1000.0;
const W_WIDTH: f64 = 1000.0;

trait Camera {
    fn world_to_screen(&self, world: Vec2d) -> Vec2d;
    fn screen_to_world(&self, screen: Vec2d) -> Vec2d;
}

impl Camera for GameObject {
    fn world_to_screen(&self, world: Vec2d) -> Vec2d {
        sub(world, self.get_pos())
    }

    fn screen_to_world(&self, screen: Vec2d) -> Vec2d {
        add(self.get_pos(), screen)
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
    velocity: Vec2d,
    rect: Option<Rect>
}

impl GameObject {
    fn new(x: f64, y: f64, color: Color, wh: Option<(f64, f64)>) -> GameObject {
        let mut rect = None;

        if let Some((hw, hh)) = wh {
            rect = Some(rectangle::centered([0., 0., hw, hh]));
        }

        GameObject {
            rotation: 0.,
            pos: Vec2d::from([x, y]),
            color: color,
            velocity: Vec2d::from([0., 0.]),
            rect: rect
        }
    }

    fn get_shape(&self) -> Option<Rect> {
        self.rect.clone()
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

    fn look_at(&mut self, target: Vec2d) -> Option<(f64, Vec2d)> {
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

        Some((angle, n_target))
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
    camera: GameObject,
    objects: Vec<GameObject>,
    players: HashMap<NetToken, Player>,
    connection: Option<Connection>,
    player_config: PlayerConfig,
    cursor: [f64; 2],
    intersects: Vec<cgmath::Point2<f64>>,
    lines: Vec<[f64; 4]>
}

struct Intersection {
    angle: f64,
    point: cgmath::Point2<f64>
}

impl Play {
    pub fn new(auto_connect: Option<String>, player_config: PlayerConfig) -> Play {
        let objects = vec![
            GameObject::new(300.0, 400.0, WHITE, Some((W_WIDTH / 2., W_HEIGHT / 2.))),
            GameObject::new(200.0, 300.0, WHITE, Some((100., 10.))),
            GameObject::new(500.0, 100.0, RED, Some((10., 100.))),
            GameObject::new(50.0, 40.0, GREEN, Some((100., 100.))),
            GameObject::new(600.0, 600.0, BLUE, Some((100., 150.))),
            GameObject::new(50.0, 500.0, BLUE, Some((50., 50.))),
            GameObject::new(50.0, 650.0, WHITE, Some((50., 50.))),
            GameObject::new(200.0, 500.0, RED, Some((50., 50.))),
            GameObject::new(200.0, 650.0, GREEN, Some((50., 50.)))
        ];

        let mut play = Play {
            switcher: BaseSwitcher::new(None),
            objects: objects,
            camera: GameObject::new(0., 0., BLUE, None),
            players: HashMap::new(),
            free_area: Rect::from([200., 150., 600., 450.]),
            connection: None,
            player_config: player_config,
            cursor: [0f64; 2],
            intersects: vec![],
            lines: vec![]
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
                }
                Err(err) => Err(err)
            },
            Err(e) => Err(format!("{:?}", e.kind()))
        }
    }

    fn spawn_player(&mut self, token: NetToken, pos: Vec2d, name: String, color: Color) {
        let idx = self.objects.len();
        self.objects.push(GameObject::new(pos[0], pos[1], color, None));

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

fn transform(dx: f64, dy: f64, rot: f64) -> cgmath::Decomposed<cgmath::Vector2<f64>, cgmath::Basis2<f64>> {
    cgmath::Decomposed {
        scale: 1.,
        rot: cgmath::Rotation2::from_angle(cgmath::Rad(rot)),
        disp: cgmath::Vector2::new(dx, dy),
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

        self.camera.update_velocity(dt);

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
            if let Some(shape) = obj.get_shape() {
                let screen_pos = self.camera.world_to_screen(obj.get_pos());
                let pos = multiply(ctx.transform, translate(screen_pos)).rot_rad(obj.rotation);
                let obj_border = Rectangle::new_border(obj.color.clone(), 0.5);
                obj_border.draw(shape, &ctx.draw_state, pos, graphics);
            }
        }

        let global_player_pos = self.player()
            .and_then(|obj| {
                Some(obj.get_pos())
            });

        let cursor = self.camera.screen_to_world(self.cursor);

        if let Some(obj) = self.player() {
            obj.look_at(cursor);
        }

        let cursor = self.cursor;

        let mut angles = vec![];
        let mut intersects = vec![];
        if let Some(global_pos) = global_player_pos {
            for obj in self.objects.iter() {
                if let Some(shape) = obj.get_shape() {
                    let collision_box = cgcoll::primitive::Rectangle::new(shape[2], shape[3]);
                    let screen_pos = self.camera.world_to_screen(obj.get_pos());
                    let corners = collision_box.get_bound().to_corners();

                    for corner in corners.iter() {
                        let corner_screen = add([corner.x, corner.y], screen_pos);
                        let n = vec2_normalized(sub(corner_screen, cursor));
                        let angle = n[1].atan2(n[0]);

                        angles.push(angle);
                        angles.push(angle - 0.00001);
                        angles.push(angle + 0.00001);
                    }
                }
            }
        }

        for angle in angles.iter() {
            let direction = [angle.cos(), angle.sin()];
            let ray = cgcoll::Ray2::new(cursor.into(), direction.into());

            let mut closest_intersect: Option<cgmath::Point2<f64>> = None;
            for obj in self.objects.iter() {
                if let Some(shape) = obj.get_shape() {
                    let collision_box = cgcoll::primitive::Rectangle::new(shape[2], shape[3]);
                    let screen_pos = self.camera.world_to_screen(obj.get_pos());
                    let transform = transform(screen_pos[0], screen_pos[1], 0.);

                    if let Some(result) = collision_box.intersection_transformed(&ray, &transform) {
                        if let Some(closest) = closest_intersect {
                            let closest_dist = closest.distance(cursor.into());
                            let dist = result.distance(cursor.into());

                            if dist < closest_dist {
                                closest_intersect = Some(result);
                            }
                        } else {
                            closest_intersect = Some(result);
                        }
                    }
                }
            }

            if let Some(closest) = closest_intersect {
                intersects.push(Intersection { angle: angle.clone(), point: closest });
            }
        }

        intersects.sort_by(|a, b| {
            b.angle.partial_cmp(&a.angle).unwrap_or(Ordering::Equal)
        });

        for (i, &Intersection { angle: _, point: p }) in intersects.iter().enumerate() {
                //if i < 6 {
                if intersects.len() > i + 1 {
                    let next = intersects[i + 1].point;
                    let polygon_points = [
                        [p.x, p.y],
                        [next.x, next.y],
                        [self.cursor[0], self.cursor[1]]
                    ];
                    polygon([0.2, 0.2, 0.2, 0.1], &polygon_points, ctx.transform.clone(), graphics);
                }

                if i == intersects.len() - 1 {
                    let first = intersects[0].point;
                    let polygon_points = [
                        [p.x, p.y],
                        [first.x, first.y],
                        [self.cursor[0], self.cursor[1]]
                    ];
                    polygon([0.2, 0.2, 0.2, 0.1], &polygon_points, ctx.transform.clone(), graphics);
                }
        }

        //self.lines.drain(..);
        //self.intersects.drain(..);

        self.player()
            .and_then(|player_obj| {
                Some((player_obj.get_pos(), player_obj.rotation))
            })
            .and_then(|(pos, rot)| {
                let screen_pos = self.camera.world_to_screen(pos);
                let player_transform = multiply(ctx.transform, translate(screen_pos)).rot_rad(rot);
                let right = player_transform.rot_rad(f64::consts::PI / 4.);
                let left = player_transform.rot_rad(-f64::consts::PI / 4.);

                line(RED, 0.5, [0., -20., 0., -40.], right, graphics);
                line(WHITE, 0.5, [0., -20., 0., -40.], player_transform, graphics);
                line(GREEN, 0.5, [0., -20., 0., -40.], left, graphics);

                Some(())
            });

        /*let area = &self.free_area;
        let rect = rectangle::rectangle_by_corners(area[0], area[1], area[2], area[3]);
        let area_border = Rectangle::new_border([0., 0., 1., 0.1], 0.5);
        area_border.draw(rect, &ctx.draw_state, ctx.transform.clone(), graphics);*/

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
                        }
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
