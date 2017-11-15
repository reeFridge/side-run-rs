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
use collision::{ContinuousTransformed, DiscreteTransformed, Contains, HasAabb};
use cgmath::MetricSpace;
use cgmath::Rotation;
use std::cmp::Ordering;
use button_tracker::ButtonController;
use asset_manager::AssetManager;

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

#[derive(Clone)]
struct GameObject {
    pos: Vec2d,
    rotation: f64,
    color: Color,
    velocity: Vec2d,
    collider: Option<Rect>,
    collides: bool
}

impl GameObject {
    fn new(x: f64, y: f64, color: Color, wh: Option<(f64, f64)>, collides: bool) -> GameObject {
        let mut rect = None;

        if let Some((hw, hh)) = wh {
            rect = Some(rectangle::centered([0., 0., hw, hh]));
        }

        GameObject {
            rotation: 0.,
            pos: Vec2d::from([x, y]),
            color: color,
            velocity: Vec2d::from([0., 0.]),
            collider: rect,
            collides
        }
    }

    fn get_collider(&self) -> Option<Rect> {
        self.collider.clone()
    }

    fn get_pos(&self) -> Vec2d {
        self.pos.clone()
    }

    fn update_position<F>(&mut self, dt: f64, collide: F)
        where F: Fn(&Vec2d) -> bool {
        if vec2_len(self.velocity) > 0.5 {
            let transform = translate(mul_scalar(self.velocity, dt));
            let friction = 0.8;
            let new_pos = transform_pos(transform, self.pos);

            if !collide(&new_pos) {
                self.pos = new_pos;
            }

            self.velocity = mul_scalar(self.velocity, friction);
        } else {
            self.velocity = Vec2d::from([0., 0.]);
        }
    }

    fn look_at(&mut self, target: Vec2d) -> Option<(f64, Vec2d)> {
        let eye = self.get_pos();
        let n = vec2_normalized(sub(eye, target));
        let angle = n[1].atan2(n[0]);

        self.rotation = angle;

        Some((angle, n))
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
    button_tracker: ButtonController,
    prev_not: Option<Vec2d>
}

struct Angle {
    rad: f64,
    color: Color
}

struct Intersection {
    angle: Angle,
    point: cgmath::Point2<f64>
}

impl Play {
    pub fn new(auto_connect: Option<String>, player_config: PlayerConfig) -> Play {
        let objects = vec![
            GameObject::new(400.0, 300.0, WHITE, Some((W_WIDTH / 2., W_HEIGHT / 2.)), false),
            GameObject::new(200.0, 300.0, WHITE, Some((100., 10.)), true),
            GameObject::new(500.0, 100.0, RED, Some((10., 100.)), true),
            GameObject::new(50.0, 40.0, GREEN, Some((100., 100.)), true),
            GameObject::new(600.0, 600.0, BLUE, Some((100., 150.)), true),
            GameObject::new(50.0, 500.0, BLUE, Some((50., 50.)), true),
            GameObject::new(50.0, 650.0, WHITE, Some((50., 50.)), true),
            GameObject::new(200.0, 500.0, RED, Some((50., 50.)), true),
            GameObject::new(200.0, 650.0, GREEN, Some((50., 50.)), true)
        ];

        let mut play = Play {
            switcher: BaseSwitcher::new(None),
            objects: objects,
            camera: GameObject::new(0., 0., BLUE, None, false),
            players: HashMap::new(),
            free_area: Rect::from([200., 150., 600., 450.]),
            connection: None,
            player_config: player_config,
            cursor: [0f64; 2],
            button_tracker: ButtonController::new(),
            prev_not: None
        };

        if let Some(addr) = auto_connect {
            match play.connect(addr) {
                Err(err) => println!("Failed to connect: {}", err),
                _ => ()
            }
        }

        play
    }

    fn render_texture(&mut self, name: &'static str, rect: Rect, transform: Matrix2d, graphics: &mut G2d, asset_manager: &mut AssetManager) {
        let image = Image::new().rect(rect);

        if let Some(texture) = asset_manager.get_texture(name) {
            image.draw(texture, &DrawState::default(), transform, graphics);
        }
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
        self.objects.push(GameObject::new(pos[0], pos[1], color, None, false));

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

    fn player_mut(&mut self) -> Option<&mut GameObject> {
        let token = match self.connection {
            Some(Connection { ref token, .. }) => token.clone(),
            None => 0 as NetToken
        };

        match self.players.get_mut(&token) {
            Some(&mut Player { obj_index: ref idx, .. }) => self.objects.get_mut(idx.clone()),
            None => None
        }
    }

    fn player(&self) -> Option<&GameObject> {
        let token = match self.connection {
            Some(Connection { ref token, .. }) => token.clone(),
            None => 0 as NetToken
        };

        match self.players.get(&token) {
            Some(&Player { obj_index: ref idx, .. }) => self.objects.get(idx.clone()),
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
        self.button_tracker.update();

        let mut colliders = self.objects.to_vec();
        colliders.retain(|obj| obj.get_collider().is_some());

        for obj in self.objects.iter_mut() {
            obj.update_position(dt, |new_pos| -> bool {
                for collider in colliders.iter() {
                    if collider.collides {
                        let rect = collider.get_collider().unwrap();
                        let bound_box = cgcoll::primitive::Rectangle::new(rect[2], rect[3]).get_bound();
                        // to local of collider transform
                        let point = cgmath::Point2::<f64>::from(sub(new_pos.clone(), collider.get_pos()));

                        if bound_box.contains(&point) {
                            return true;
                        }
                    }
                }

                false
            });
        }

        self.camera.update_position(dt, |_| false);

        if self.player().is_some() {
            let movement_keys = [
                Key::Up,
                Key::Down,
                Key::Left,
                Key::Right
            ];

            for key in movement_keys.iter() {
                let key_pressed = self.button_tracker.current_pressed(&Button::Keyboard(key.clone()));

                if key_pressed {
                    self.player_mut().unwrap().move_to(Vec2d::from(Direction::from(key.clone())), 200.);
                }
            }
        }

        // update camera pos
        {
            let screen_vec = if let Some(obj) = self.player() {
                self.camera.world_to_screen(obj.get_pos())
            } else {
                self.cursor
            };

            let center = cgmath::Point2::<f64>::from([400., 300.]);
            let screen_x = cgmath::Point2::<f64>::from([screen_vec[0], center.y]);
            let screen_y = cgmath::Point2::<f64>::from([center.x, screen_vec[1]]);
            let mut direction = Vec2d::from([0., 0.]);

            if center.distance(screen_x) > 150. {
                direction = add(direction, [screen_x.x - center.x, 0.]);
            }

            if center.distance(screen_y) > 150. {
                direction = add(direction, [0., screen_y.y - center.y]);
            }

            self.camera.move_to(vec2_normalized(direction), 200.);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, graphics: &mut G2d, asset_manager: &mut AssetManager) -> GameResult<()> {
        clear(BLACK, graphics);

        let pos = self.camera.world_to_screen([-100., -200.]);
        let (tile_width, tile_height) = (200., 200.);
        let iter_x = (W_WIDTH / tile_width) as i32;
        let iter_y = (W_HEIGHT / tile_height) as i32;
        let t = ctx.transform.trans(pos[0], pos[1]);
        for i in 0..iter_x {
            for j in 0..iter_y {
                let rect = [0. + tile_width * i as f64, 0. + tile_height * j as f64, tile_width, tile_height];
                self.render_texture("floor", rect, t, graphics, asset_manager);
            }
        }

        // turn texture to black
        let pos = self.camera.world_to_screen([400., 300.]);
        let t = ctx.transform.trans(pos[0], pos[1]);
        let rect = rectangle::centered([0., 0., W_WIDTH / 2., W_HEIGHT / 2.]);
        rectangle([0., 0., 0., 0.96], rect, t, graphics);

        for obj in self.objects.iter() {
            if let Some(collider) = obj.get_collider() {
                let screen_pos = self.camera.world_to_screen(obj.get_pos());
                let pos = multiply(ctx.transform, translate(screen_pos)).rot_rad(obj.rotation);
                let obj_border = Rectangle::new_border(obj.color.clone(), 0.5);
                obj_border.draw(collider, &ctx.draw_state, pos, graphics);
            }
        }

        let cursor = self.camera.screen_to_world(self.cursor);

        if let Some(obj) = self.player_mut() {
            obj.look_at(cursor);
        }

        // Shadow-casting
        {
            let source = if let Some(obj) = self.player() {
                self.camera.world_to_screen(obj.get_pos())
            } else {
                self.cursor
            };

            let rotation = if let Some(obj) = self.player() {
                obj.rotation.clone()
            } else {
                0.
            };

            fn add_auxiliary_angles(current: &cgmath::Point2<f64>, prev: &cgmath::Point2<f64>, next: &cgmath::Point2<f64>, angles: &mut Vec<Angle>, aux_angle: f64, angle: f64) {
                let direction = [angle.cos(), angle.sin()];
                let prev_vec = sub(current.clone().into(), prev.clone().into());
                let next_vec = sub(next.clone().into(), current.clone().into());
                let projection_axis = perp(direction);
                let p_1 = vec2_dot(prev_vec, projection_axis);
                let p_2 = vec2_dot(next_vec, projection_axis);

                if p_1 >= 0. && p_2 <= 0. {
                    // counter-clockwise
                    angles.push(Angle { rad: angle + aux_angle, color: GREEN });
                } else if p_1 <= 0. && p_2 >= 0. {
                    // clockwise
                    angles.push(Angle { rad: angle - aux_angle, color: BLUE });
                }
            }

            let mut angles = vec![];
            let mut intersects = vec![];
            for obj in self.objects.iter() {
                if let Some(collider) = obj.get_collider() {
                    let collision_box = cgcoll::primitive::Rectangle::new(collider[2], collider[3]);
                    let screen_pos = self.camera.world_to_screen(obj.get_pos());
                    let corners = collision_box.get_bound().to_corners();

                    for (i, corner) in corners.iter().enumerate() {
                        let prev_index = if i == 0 {
                            corners.len() - 1
                        } else {
                            i - 1
                        };

                        let next_index = if i + 1 == corners.len() {
                            0
                        } else {
                            i + 1
                        };

                        let ref prev = corners[prev_index];
                        let ref next = corners[next_index];

                        let corner_screen = add(corner.clone().into(), screen_pos);
                        let n = vec2_normalized(sub(corner_screen, source));
                        let angle = n[1].atan2(n[0]);

                        angles.push(Angle { rad: angle, color: RED });
                        add_auxiliary_angles(corner, prev, next, &mut angles, 0.01, angle);
                    }
                }
            }

            for angle in angles.iter() {
                let direction = [angle.rad.cos(), angle.rad.sin()];
                let ray = cgcoll::Ray2::new(source.into(), direction.into());

                let mut closest_intersect: Option<cgmath::Point2<f64>> = None;
                for obj in self.objects.iter() {
                    if let Some(collider) = obj.get_collider() {
                        let collision_box = cgcoll::primitive::Rectangle::new(collider[2], collider[3]);
                        let screen_pos = self.camera.world_to_screen(obj.get_pos());
                        let transform = transform(screen_pos[0], screen_pos[1], 0.);

                        let result = collision_box.intersection_transformed(&ray, &transform)
                            .or_else(|| {
                                // Dummy correction for ray intersection
                                for (i, corner) in collision_box.get_bound().to_corners().iter().enumerate() {
                                    let corner_screen = add(corner.clone().into(), screen_pos);
                                    let n = vec2_normalized(sub(corner_screen, source));
                                    let _angle = n[1].atan2(n[0]);

                                    if angle.rad == _angle {
                                        return Some(cgmath::Point2::from(corner_screen));
                                    }
                                }

                                None
                            });

                        if let Some(result) = result {
                            if let Some(closest) = closest_intersect {
                                let closest_dist = closest.distance(source.into());
                                let dist = result.distance(source.into());

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
                    intersects.push(Intersection { angle: Angle { color: angle.color, rad: angle.rad }, point: closest });
                    /* Some tries to decrease counts of intersection points
                       idea: deal only with closest corners

                    'objects: for obj in self.objects.iter() {
                        if let Some(collider) = obj.get_collider() {
                            let collision_box = cgcoll::primitive::Rectangle::new(collider[2], collider[3]);
                            let screen_pos = self.camera.world_to_screen(obj.get_pos());
                            let corners = collision_box.get_bound().to_corners();

                            'corners: for corner in corners.iter() {
                                let corner_screen = add([corner.x, corner.y], screen_pos);

                                if closest.distance(corner_screen.into()) < 0.5 {
                                    intersects.push(Intersection { angle: angle.clone(), point: closest });
                                    break 'objects;
                                }
                            }
                        }
                    }*/
                }
            }

            intersects.sort_by(|a, b| {
                b.angle.rad.partial_cmp(&a.angle.rad).unwrap_or(Ordering::Equal)
            });

            let grey = [0.2, 0.2, 0.2, 0.2];
            let polygon = Polygon::new(grey);
            for (i, &Intersection { angle: ref angle, point: p }) in intersects.iter().enumerate() {
                let line = Line::new(angle.color.clone(), 0.5);
                if intersects.len() > i + 1 {
                    let next = intersects[i + 1].point;
                    let polygon_points = [
                        [p.x, p.y],
                        [next.x, next.y],
                        [source[0], source[1]]
                    ];
                    polygon.draw_tri(&polygon_points, &ctx.draw_state, ctx.transform.clone(), graphics);
                }

                // last triangle end-start
                if i == intersects.len() - 1 {
                    let first = intersects[0].point;
                    let polygon_points = [
                        [p.x, p.y],
                        [first.x, first.y],
                        [source[0], source[1]]
                    ];
                    polygon.draw_tri(&polygon_points, &ctx.draw_state, ctx.transform.clone(), graphics);
                }

                //line.draw([source[0], source[1], p.x, p.y], &ctx.draw_state, ctx.transform.clone(), graphics);
                //rectangle(WHITE, rectangle::centered_square(p.x, p.y, 2.), ctx.transform.clone(), graphics);
            }
        }

        if self.player().is_some() {
            let rot = self.player().unwrap().rotation.clone();
            let pos = self.player().unwrap().get_pos();
            let screen_pos = self.camera.world_to_screen(pos);
            let player_transform = multiply(ctx.transform, translate(screen_pos));

            let rect = rectangle::centered_square(0.0, 0.0, 50.0);
            self.render_texture("player_sprite", rect, player_transform.rot_rad(rot), graphics, asset_manager);

            // may be FOV in the nearest future
            let ellipse = Ellipse::new_border(BLUE, 0.5);
            let circle = ellipse::circle(0.0, 0.0, 200.0);
            ellipse.draw(circle, &ctx.draw_state, player_transform, graphics);
        }

        Ok(())
    }

    fn key_press(&mut self, button: Button) {
        self.button_tracker.register_press(&button);

        if self.player().is_none() {
            if let Button::Keyboard(key) = button {
                match key {
                    Key::Space => {
                        let spawn_pos = Vec2d::from([400., 300.]);
                        self.spawn_self_player(spawn_pos);
                    }
                    _ => ()
                };
            }
        }
    }

    fn key_release(&mut self, button: Button) {
        self.button_tracker.register_release(&button);
    }

    fn mouse_move(&mut self, cursor: [f64; 2]) {
        self.cursor = cursor;
    }
}
