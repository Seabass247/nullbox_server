pub mod client;
pub mod datatypes;
pub mod server;

#[macro_use]
extern crate gdnative as godot;
extern crate bincode;
extern crate crossbeam_channel;
extern crate laminar;
extern crate nullbox_core as nullbox;
extern crate serde_derive;

use bincode::{deserialize, serialize};
use client::Client;
use crossbeam_channel::{Receiver, Sender};
use laminar::{ErrorKind, Packet, Socket, SocketEvent};
use datatypes::{VariantType, VariantTypes};
use serde_derive::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::net::{IpAddr, Ipv4Addr};
use std::{thread, time};
use server::Server;

struct Laminar {
    client: Option<Client>,
    server: Option<Server>,
    callback: bool,
}

fn get_available_port() -> Option<u16> {
    (8000..9000).find(|port| match Socket::bind(("127.0.0.1", *port)) {
        Ok(_) => true,
        Err(_) => false,
    })
}

#[gdnative::methods]
impl Laminar {
    fn _init(_owner: gdnative::Node) -> Self {
        godot_print!("Laminar: plugin initialized!");
        Laminar {
            client: None,
            server: None,
            callback: false,
        }
    }

    #[export]
    fn _ready(&self, _owner: gdnative::Node) {
        godot_print!("Laminar: plugin ready");
    }

    #[export]
    fn test(&mut self, _owner: gdnative::Node, player_id: i64, destination: godot::GodotString, variant: godot::VariantArray) {
        godot_print!("Laminar: test: {}", player_id.to_string());
    }

    // Client only func
    #[export]
    fn send(&mut self, _owner: gdnative::Node, destination: godot::GodotString, variant: godot::VariantArray) {
        let variant = godot::Variant::from_array(&variant);
        let dest = &destination.to_string();
        let dest_split: Vec<&str> = dest.split(":").collect();
        let node_path = dest_split[0];
        // Fail if we dont get a destination formatted "$NODE_PATH:$METHOD"
        if dest_split.len() <= 1 {
            godot_print!("Laminar: error trying to parse send destination path");
            return;
        }
        let method = dest_split[1];
        match self.client.take() {
            Some(mut client) => {
                client.send_vars(node_path.to_string(), method.to_string(), VariantTypes::from(variant));
                //godot_print!("Laminar: send var packet");
                self.client = Some(client);
            }
            None => {
                godot_print!("Laminar error: must initialize client before sending data");
            }
        }
    }

    // Client only func
    #[export]
    fn sleep(&mut self, _owner: gdnative::Node, time: i64) {
        match self.client.take() {
            Some(mut client) => {
                let time = std::time::Duration::from_millis(time as u64);
                client.recv_sleep.0.send(time);
                self.client = Some(client);
            }
            None => {
                godot_print!("Laminar error: must initialize client first");
            }
        }
    }

    // Server only func
    #[export]
    fn send_to(&mut self, _owner: gdnative::Node, player_id: i64, destination: godot::GodotString, variant: godot::VariantArray) {
        let variant = godot::Variant::from_array(&variant);
        let dest = &destination.to_string();
        let dest_split: Vec<&str> = dest.split(":").collect();
        let node_path = dest_split[0];
        // Fail if we dont get a destination formatted "$NODE_PATH:$METHOD"
        if dest_split.len() <= 1 {
            godot_print!("Laminar: error trying to parse send destination path");
            return;
        }
        let method = dest_split[1];
        match player_id {
            0 => {
                match self.server.clone() {
                    Some(mut server) => {
                        server.send_to_all(node_path.to_string(), method.to_string(), VariantTypes::from(variant));
                        //godot_print!("Laminar Server: send var packet to all");
                        self.server = Some(server);
                    }
                    None => {
                        godot_print!(
                            "Laminar error: must initialize server before sending data"
                        );
                    }
                }         
            }
            _ => {
                match self.server.clone() {
                    Some(mut server) => {
                        server.send_to(player_id, node_path.to_string(), method.to_string(), VariantTypes::from(variant));
                        //godot_print!("Laminar Server: send var packet to {}", player_id);
                        self.server = Some(server);
                    }
                    None => {
                        godot_print!(
                            "Laminar error: must initialize server before sending data"
                        );
                    }
                }
            }
        }
    }

    /// Client only func
    #[export]
    fn init_client(&mut self, _owner: gdnative::Node, address: godot::GodotString) {
        // setup an udp socket and bind it to the client address.
        if self.client.is_some() {
            let mut client = self.client.take().unwrap();
            let server_address: SocketAddr = address.to_string().parse().unwrap();
            client.server_address = server_address;
            self.client = Some(client);
            return;
        }

        let mut client_socket = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            get_available_port().unwrap(),
        );
        let (mut socket, packet_sender, event_receiver) = Socket::bind(client_socket).unwrap();
        let polling_thread = thread::spawn(move || socket.start_polling());

        let server_address: SocketAddr = address.to_string().parse().unwrap();

        // For telling the recv data thread to block for a time
        let (tx_sleep, rx_sleep): (Sender<std::time::Duration>, Receiver<std::time::Duration>) = crossbeam_channel::unbounded();

        let client = Client {
            packet_sender,
            _event_receiver: event_receiver,
            server_address,
            uid: None,
            recv_sleep: (tx_sleep, rx_sleep),
        };

        self.client = Some(client);

        match self.client.clone() {
            Some(client) => unsafe {
                client.start_receiving(_owner);
                godot_print!(
                    "Laminar: client waiting for connection response from server {}",
                    address.to_string()
                );
            },
            None => {}
        }
    }

    /// Server only func
    #[export]
    fn init_server(&mut self, _owner: gdnative::Node, port: godot::GodotString) {
        let server = Server::new(port.to_string());
        self.server = Some(server);

        match self.server.clone() {
            Some(server) => unsafe {
                server.start_receiving(_owner);
                godot_print!("Laminar: server waiting for connections");
            },
            None => {}
        }
    }
}

impl godot::NativeClass for Laminar {
    type Base = godot::Node;

    fn class_name() -> &'static str {
        "Laminar"
    }

    fn init(_owner: Self::Base) -> Self {
        Self::_init(_owner)
    }

    fn register_properties(builder: &godot::init::ClassBuilder<Self>) {
        builder.add_signal(godot::init::Signal {
            name: "recv_data",
            args: &[],
        });
    }
}

fn init(handle: gdnative::init::InitHandle) {
    handle.add_class::<Laminar>();
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();
