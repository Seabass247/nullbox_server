use nullbox_core::DataType;
use std::net::SocketAddr;
use std::fmt;

pub struct Player {
    pub ip: SocketAddr,
    pub username: String,
    pub id: String,
    pub pos: Option<Position>,
}

pub struct Position {
    pos_x: f32, 
    pos_y: f32,
    pos_z: f32,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{},{}", self.pos_x, self.pos_y, self.pos_z)
    }
}