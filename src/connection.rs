use std::net::TcpStream;
use byteorder::{ByteOrder, BigEndian};
use std::time::Duration;
use scenes::common::*;
use std::io::{Read, Write};
use piston_window::types::Color;
use piston_window::math::Vec2d;

pub type NetToken = usize;

pub struct Connection {
    pub socket: TcpStream,
    pub token: NetToken
}

pub enum EventType {
    Spawn,
    UpdatePos
}

impl Connection {
    pub fn new(mut socket: TcpStream) -> Result<Connection, String> {
        let mut buf = [0u8; 8];

        match socket.read(&mut buf) {
            Ok(_) => Ok(Connection {
                socket: socket,
                token: BigEndian::read_u64(&buf) as usize
            }),
            Err(e) => Err(format!("{:?}", e.kind()))
        }
    }

    pub fn listen_events(&mut self) -> Option<(EventType, String)> {
        let mut buf = [0u8; 64];
        self.socket.set_read_timeout(Some(Duration::from_millis(10))).unwrap();

        match self.socket.read(&mut buf) {
            Ok(_) => {
                let (event, raw_data) = buf.split_at(5);
                let data = String::from_utf8_lossy(&raw_data).into_owned();

                match Connection::parse_event_type(&event) {
                    Some(e) => Some((e, data)),
                    None => None
                }
            },
            Err(_) => None
        }
    }

    pub fn send_spawn_event(&mut self, name: String, pos: Vec2d, color: Color) -> Result<(), String> {
        let token = self.token.clone();
        let x = pos[0];
        let y = pos[1];

        let u32_color = {
            let to_255 = 255f32;
            let r = color[0] * to_255;
            let g = color[1] * to_255;
            let b = color[2] * to_255;
            let a = color[3] * to_255;

            let rp = (r as u32) << 24;
            let gp = (g as u32) << 16;
            let bp = (b as u32) << 8;
            let ap = a as u32;

            (rp | gp | bp | ap)
        };


        self.socket.write_all(format!("SPWN {}|{}|{}x{}|{}\r\n", token, name, x, y, u32_color).as_bytes()).unwrap();
        self.socket.flush().unwrap();

        Ok(())
    }

    pub fn send_update_pos_event(&mut self, pos: Vec2d) -> Result<(), String> {
        let x = pos[0];
        let y = pos[1];
        let token = self.token.clone();
        self.socket.write_all(format!("UPDP {}|{}x{}\r\n", token, x, y).as_bytes()).unwrap();
        self.socket.flush().unwrap();

        Ok(())
    }

    pub fn parse_event_type(buf: &[u8]) -> Option<EventType> {
        let cow_str = String::from_utf8_lossy(buf).into_owned();
        let (event_name, _) = cow_str.as_str().split_at(4);

        match event_name {
            "SPWN" => Some(EventType::Spawn),
            "UPDP" => Some(EventType::UpdatePos),
            _ => None
        }
    }

    pub fn parse_update_pos_event(data: String) -> Result<(usize, Vec2d), String> {
        let str: Vec<&str> = data.as_str().split("\r\n").collect();
        let data_str = str[0].trim();
        let data_parts: Vec<&str> = data_str.split("|").collect();
        let token = data_parts[0].parse::<u64>().expect("token") as usize;
        let coords: Vec<&str> = data_parts[1].split("x").collect();
        let pos = Vec2d::from([
            coords[0].parse::<f64>().expect("x"),
            coords[1].parse::<f64>().expect("y")
        ]);

        Ok((token, pos))
    }

    pub fn parse_spawn_event(data: String) -> Result<(usize, String, Vec2d, Color), String> {
        let str: Vec<&str> = data.as_str().split("\r\n").collect();
        let data_str = str[0].trim();
        let data_parts: Vec<&str> = data_str.split("|").collect();
        let token = data_parts[0].parse::<u64>().expect("token") as usize;
        let name = data_parts[1].to_string();
        let coords: Vec<&str> = data_parts[2].split("x").collect();
        let pos = Vec2d::from([
            coords[0].parse::<f64>().expect("x"),
            coords[1].parse::<f64>().expect("y")
        ]);
        let color_u = data_parts[3].parse::<u32>().expect("color");

        let inv_255 = 1.0f32 / 255.0f32;
        let color = {
            let rp = (color_u >> 24) as u8;
            let gp = (color_u >> 16) as u8;
            let bp = (color_u >> 8) as u8;
            let ap = color_u as u8;

            [rp as f32 * inv_255, gp as f32 * inv_255, bp as f32 * inv_255, ap as f32 * inv_255]
        };

        Ok((token, name, pos, color))
    }
}
