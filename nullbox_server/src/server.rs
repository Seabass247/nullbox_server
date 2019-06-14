use crate::message;
use crate::player::{Player, Position};
use bincode::{deserialize, serialize};
use crossbeam_channel::{Receiver, Sender};
use laminar::{ErrorKind, Packet, Socket, SocketEvent};
use nullbox_core::DataType;
use serde_derive::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::{thread, time};
use super::HashMap;

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

    pub fn start_packet_handling(&self, s: Sender<message::Event>) {
        let er = self.event_receiver.clone();
        let ps = self.packet_sender.clone();

        // Start the listening thread.
        thread::spawn(move || loop {
            receive_packet(&er, &ps, &s);
        });
    }

    pub fn send_all(&self, send_buf: crate::MsgsToSend) {
        let send_buf = send_buf.clone();
        let packet_sender = self.packet_sender.clone();
        thread::spawn(move || {
            for (addr, msg_str) in send_buf {
                &packet_sender
                    .send(Packet::reliable_unordered(addr, msg_str.as_bytes().to_vec()))
                    .unwrap();
            }
        });
    }

    pub fn send_all_ordered(&self, send_buf: crate::MsgsToSend) {
        let send_buf = send_buf.clone();
        let packet_sender = self.packet_sender.clone();
        thread::spawn(move || {
            for (addr, msg_str) in send_buf {
                &packet_sender
                    .send(Packet::reliable_ordered(addr, msg_str.as_bytes().to_vec(), None))
                    .unwrap();
            }
        });
    }
}

/// Receive and block the current thread.
fn receive_packet(
    event_receiver: &Receiver<SocketEvent>,
    packet_sender: &Sender<Packet>,
    s: &Sender<message::Event>,
) {
    // Next start receiving.
    let result = event_receiver.recv();

    match result {
        Ok(SocketEvent::Packet(packet)) => {
            let received_data: &[u8] = packet.payload();
            //println!("Received data {:?}", packet.payload());

            let msg = std::str::from_utf8(received_data).unwrap().to_owned();
            let s = s.clone();

            // Parse recv'd message a new thread so we can continue receiving packets in this thread
            thread::spawn(move || {
                handle_message(&msg, &s, packet.addr());
            });
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

/// Interface server game data from our received msg string.
fn handle_message(msg: &str, sender: &Sender<message::Event>, pack_addr: SocketAddr) {
    match message::parse_msg_str(msg) {
        // TODO: move parsing logic in an impl of Event???
        Some(msg) => match msg.head.as_str() {
            "regplr" => {
                sender.send(message::Event::RegNewPlayer {
                    username: msg.body[0].clone(),
                    address: pack_addr,
                });
            }
            "plrmov" => {
                sender.send(message::Event::PlayerMove {
                    id: msg.body[0].parse::<i32>().unwrap(),
                    new_pos: Position {
                        x: msg.body[1].parse::<f32>().unwrap(),
                        y: msg.body[2].parse::<f32>().unwrap(),
                        z: msg.body[3].parse::<f32>().unwrap(),
                    },
                });
            }
            _ => {
                println!("Got an unrecognized message head '{}'", msg.head.as_str());
            }
        },
        None => {
            println!("Couldn't parse message from packet");
        }
    }
}
