use bincode::{deserialize, serialize};
use crossbeam_channel::{Receiver, Sender};
use laminar::{ErrorKind, Packet, Socket, SocketEvent};
use serde_derive::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::{thread, time};
use nullbox_core::DataType;
use crate::player::Player;

const SERVER_ADDR: &'static str = "127.0.0.1:12345";

pub fn server_address() -> SocketAddr {
    SERVER_ADDR.parse().unwrap()
}

pub struct Server {
    packet_sender: Sender<Packet>,
    event_receiver: Receiver<SocketEvent>,
    _polling_thread: thread::JoinHandle<Result<(), ErrorKind>>,
    players: Vec<Player>,
}

impl Server {
    pub fn new() -> Self {
        // setup an udp socket and bind it to the client address.
        let (mut socket, packet_sender, event_receiver) = Socket::bind(server_address()).unwrap();
        let polling_thread = thread::spawn(move || socket.start_polling());
        Server {
            packet_sender,
            event_receiver,
            _polling_thread: polling_thread,
            players: Vec::new(),
        }
    }

    /// Receive and block the current thread.
    pub fn receive(&mut self) {
        // Next start receiving.
        let result = self.event_receiver.recv();

        match result {
            Ok(SocketEvent::Packet(packet)) => {
                let received_data: &[u8] = packet.payload();
                //println!("Received data {:?}", packet.payload());
                let deserialized: DataType = deserialize(&received_data).unwrap();

                self.perform_action(deserialized, packet.addr());

                self.packet_sender
                    .send(Packet::reliable_unordered(
                        packet.addr(),
                        "Copy that!".as_bytes().to_vec(),
                    ))
                    .unwrap();
            }
            Ok(SocketEvent::Timeout(address)) => {
                println!("A client timed out: {}", address);
            }
            Ok(_) => {}
            Err(e) => {
                println!("Something went wrong when receiving, error: {:?}", e);
            }
        }
    }

    /// Perform some processing of the data we have received.
    fn perform_action(&mut self, data_type: DataType, pack_addr: SocketAddr) {
        match data_type {
            DataType::Coords {
                x,
                y,
                z,
            } => {
                println!(
                    
                    "Moving to x: {}, y: {}, z: {}",
                    x, y, z
                );
            }
            DataType::ASCII { string } => {
                println!("Received text: {:?}", string);
                let mut split = string.split(":");
                let head = split.next();
                let body: Vec<&str> = match split.next() {
                    Some(body) => {
                        body.split(",").collect()
                    }
                    _ => { Vec::new() }
                };
                match head {
                    Some(head) => {
                        match head {
                            "reg" => {
                                let player = Player {
                                        ip: pack_addr,
                                        username: body[0].to_string(),
                                        id: body[1].to_string(),
                                        pos: None,
                                };
                                self.players.push(player);
                                println!("Registered player with name {}, id {}", body[0], body[1]);
                                self.packet_sender
                                    .send(Packet::reliable_unordered(
                                        pack_addr,
                                        "reg:success".as_bytes().to_vec(),
                                    ))
                                    .unwrap();
                            }
                            _ => {

                            }
                        }
                    }
                    _ => {
                        println!("Packet has no head");
                    }
                }
            }
            DataType::Transform { .. } => {

            }
        }
    }
}
