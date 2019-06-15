mod message;
mod player;
mod server;
use crate::server::Server;
use bincode::{deserialize, serialize};
use crossbeam_channel::{unbounded, Receiver, Sender};
use laminar::{ErrorKind, Packet, Socket, SocketEvent};
use message::Event;
use player::Player;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::{thread, time};

type MsgsToSend = Vec<(SocketAddr, String)>;

fn main() {
    println!(
        "NullBox Server. Listening for connections on {}",
        server::server_address()
    );
    let mut unique_ids: Vec<i32> = Vec::new();
    let mut players: HashMap<SocketAddr, Player> = HashMap::new();

    let mut server = Server::new();
    // set up or `Server` that will receive the messages we send with the `Client`
    let (s1, r1) = unbounded();
    server.start_packet_handling(s1);
    // Main game loop
    loop {
        let mut send_buf: MsgsToSend = Vec::new();

        match r1.try_recv() {
            Ok(event) => {
                match event {
                    Event::RegNewPlayer { username, address } => {
                        // Get a unique new id for this player's connection instance.
                        let id: i32 = unique_ids.len() as i32 + 1;
                        unique_ids.push(id);

                        println!(
                            "Main loop: Register new player \"{}\" with id: {}",
                            username, id
                        );
                        let mut new_player = Player {
                            ip: address,
                            username,
                            id,
                            pos: None,
                        };
                        // Add the server response to the send buffer.
                        let to_send = format!("0#/root/MainMenu>reg_success;{}", id);
                        send_buf.push((address, to_send));
                        // Add the player to a dict of players associated with their unique id.
                        players.insert(address, new_player);
                    }
                    Event::PlayerMove { address, new_pos } => {
                        if let Some(plr) = players.get_mut(&address) {
                            plr.pos = Some(new_pos);
                        }
                    }
                    Event::PlayerDisconnect { address } => {
                        players.remove_entry(&address);
                    }
                }
            }
            _ => {}
        }

        for (id, player) in &players {
            if let Some(pos) = &player.pos {
                let player_locations: String = players
                    .values()
                    .filter(|p| p.pos.is_some())
                    .map(|p| format!("{}={};", p.id, p.pos.clone().unwrap()))
                    .collect();
                let to_send = format!("0#/root/Game>upd_ply;{}", player_locations);
                send_buf.push((player.ip, to_send));
                println!("locations {}", player_locations);
            }
        }

        server.send_all(send_buf);

        std::thread::sleep_ms(10);
    }
}
