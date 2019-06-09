use bincode::{deserialize, serialize};
use crossbeam_channel::{Receiver, Sender};
use laminar::{ErrorKind, Packet, Socket, SocketEvent};
use serde_derive::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::{thread, time};
use nullbox_core::DataType;

const SERVER_ADDR: &'static str = "127.0.0.1:12345";

pub fn server_address() -> SocketAddr {
    SERVER_ADDR.parse().unwrap()
}

pub struct Server {
    _packet_sender: Sender<Packet>,
    event_receiver: Receiver<SocketEvent>,
    _polling_thread: thread::JoinHandle<Result<(), ErrorKind>>,
}

impl Server {
    pub fn new() -> Self {
        // setup an udp socket and bind it to the client address.
        let (mut socket, packet_sender, event_receiver) = Socket::bind(server_address()).unwrap();
        let polling_thread = thread::spawn(move || socket.start_polling());
        Server {
            _packet_sender: packet_sender,
            event_receiver,
            _polling_thread: polling_thread,
        }
    }

    /// Receive and block the current thread.
    pub fn receive(&mut self) {
        // Next start receiving.
        let result = self.event_receiver.recv();

        match result {
            Ok(SocketEvent::Packet(packet)) => {
                let received_data: &[u8] = packet.payload();
                println!("Received data {:?}", packet.payload());
                let deserialized: DataType = deserialize(&received_data).unwrap();

                self.perform_action(deserialized);
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
    fn perform_action(&self, data_type: DataType) {
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
            }
        }
    }
}
