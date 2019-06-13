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

fn main() {
    println!(
        "NullBox Server. Listening for connections on {}",
        server::server_address()
    );
    let mut unique_ids: Vec<i32> = Vec::new();
    let mut players: HashMap<i32, Player> = HashMap::new();

    let mut server = Server::new();
    // set up or `Server` that will receive the messages we send with the `Client`
    let (s1, r1) = unbounded();
    server.start_packet_handling(s1);
    // Main game loop
    loop {
        //println!("LOOP!");
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
                            pos: player::Position {
                                x: 0.0,
                                y: 0.0,
                                z: 0.0,
                            },
                        };
                        // Add the player to a dict of players associated with their unique id.
                        players.insert(id, new_player);
                    }
                    Event::PlayerMove { id, new_pos } => {
                        println!("Main loop: Move a player");
                    }
                }
            }
            _ => {}
        }

        for (id, player) in &players {
            let player_locations: String = players
                .values()
                .map(|p| format!("{}={};", p.id, p.pos.to_string()))
                .collect();
            println!("locations {}", player_locations)
        }

        std::thread::sleep_ms(10);
    }
}
