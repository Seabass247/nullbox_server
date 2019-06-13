use crate::player;
use std::net::SocketAddr;

pub struct Message {
    pub head: String,
    pub body: Vec<String>,
}

/// Return the message head if there exists the head delimiter ":"
pub fn parse_msg_str(msg: &str) -> Option<Message> {
    let split: Vec<&str> = msg.split(":").collect();
    if split.len() > 1 {
        let head = split[0].to_string();
        let body: Vec<String> = split[1].split("&").map(|s| s.to_string()).collect();
        Some(Message { head, body })
    } else {
        None
    }
}

/// Represents any type of event or update a player can send to the server
pub enum Event {
    RegNewPlayer {
        username: String,
        address: SocketAddr,
    },
    PlayerMove {
        id: i32,
        new_pos: player::Position,
    },
}
