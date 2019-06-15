use nullbox_core::DataType;
use std::fmt;
use std::net::SocketAddr;

pub struct Player {
    pub ip: SocketAddr,
    pub username: String,
    pub id: i32,
    pub pos: Option<Position>,
}

#[derive(Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{},{}", self.x, self.y, self.z)
    }
}
