use std::net::TcpStream;
use byteorder::{ByteOrder, BigEndian};
use ggez::graphics::{Point, Color};
use std::io::{Read, Write};

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

    pub fn send_spawn_event(&mut self, name: String, pos: Point, color: Color) -> Result<(), String> {
        let token = self.token.clone();
        self.socket.write_all(format!("SPWN {}|{}|{}x{}|{}\r\n", token, name, pos.x, pos.y, u32::from(color)).as_bytes()).unwrap();
        self.socket.flush().unwrap();

        Ok(())
    }

    pub fn send_update_pos_event(&mut self, pos: Point) -> Result<(), String> {
        let token = self.token.clone();
        self.socket.write_all(format!("UPDP {}|{}x{}\r\n", token, pos.x, pos.y).as_bytes()).unwrap();
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

    pub fn parse_update_pos_event(buf: &[u8]) -> Result<(usize, Point), String> {
        let cow_str = String::from_utf8_lossy(buf).into_owned();
        let str: Vec<&str> = cow_str.as_str().split("\r\n").collect();
        let data_str = str[0].trim();
        let data_parts: Vec<&str> = data_str.split("|").collect();
        let token = data_parts[0].parse::<u64>().expect("token") as usize;
        let coords: Vec<&str> = data_parts[1].split("x").collect();
        let pos = Point::new(coords[0].parse::<f32>().expect("x"), coords[1].parse::<f32>().expect("y"));

        Ok((token, pos))
    }

    pub fn parse_spawn_event(buf: &[u8]) -> Result<(usize, String, Point, Color), String> {
        let cow_str = String::from_utf8_lossy(buf).into_owned();
        let str: Vec<&str> = cow_str.as_str().split("\r\n").collect();
        let data_str = str[0].trim();
        let data_parts: Vec<&str> = data_str.split("|").collect();
        let token = data_parts[0].parse::<u64>().expect("token") as usize;
        let name = data_parts[1].to_string();
        let coords: Vec<&str> = data_parts[2].split("x").collect();
        let pos = Point::new(coords[0].parse::<f32>().expect("x"), coords[1].parse::<f32>().expect("y"));
        let color_u = data_parts[3].parse::<u32>().expect("color");
        let color = {
            let rp = (color_u >> 24) as u8;
            let gp = (color_u >> 16) as u8;
            let bp = (color_u >> 8) as u8;
            let ap = color_u as u8;

            Color::from((rp, gp, bp, ap))
        };

        Ok((token, name, pos, color))
    }
}
