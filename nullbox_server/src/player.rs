use nullbox_core::DataType;
use std::net::SocketAddr;

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