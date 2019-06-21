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
    client_heartbeat_time: f64,
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
            client_heartbeat_time: 0.0,
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
        match self.client.clone() {
            Some(mut client) => {
                let time = std::time::Duration::from_millis(time as u64);
                // Send sleep = true to the recv thread, wait for `time` ms, and then send sleep = false,
                thread::spawn(move || {
                    client.recv_sleep.0.send(true);
                    std::thread::sleep(time);
                    client.recv_sleep.0.send(false);
                });
            }
            None => {
                godot_print!("Laminar error: must initialize client first");
            }
        }
    }

    // Client only func
    #[export]
    fn set_root(&mut self, _owner: gdnative::Node, root: godot::GodotString) {
        match self.client.clone() {
            Some(mut client) => {
                // Send the scene root path to the recv thread.
                thread::spawn(move || {
                    client.current_root.0.send(root.to_string());
                });
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

    // For Laminar client only.  Sends a heartbeat to the server so it's connection won't time out.
    #[export]
    unsafe fn _physics_process(&mut self, mut _owner: godot::Node, delta: f64) {
        if let Some(client) = self.client.as_mut() {
            self.client_heartbeat_time += delta;
            if self.client_heartbeat_time > 3.0 {
                self.client_heartbeat_time = 0.0;
                client.send_sync(datatypes::MetaMessage::Heartbeat);
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
        let (tx_sleep, rx_sleep): (Sender<bool>, Receiver<bool>) = crossbeam_channel::unbounded();
        let (tx_root, rx_root): (Sender<String>, Receiver<String>) = crossbeam_channel::unbounded();

        let client = Client {
            packet_sender,
            _event_receiver: event_receiver,
            server_address,
            uid: None,
            recv_sleep: (tx_sleep, rx_sleep),
            current_root: (tx_root, rx_root)
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
    fn init_server(&mut self, _owner: gdnative::Node, context: godot::Node, port: godot::GodotString) {
        let server = Server::new(_owner, port.to_string());
        self.server = Some(server);

        match self.server.clone() {
            Some(server) => unsafe {
                server.start_receiving(_owner);
                godot_print!("Laminar: server waiting for connections");
                // Connect the timed out signal to the calling gdscript
                let object = &context.to_object();
                _owner
                    .clone()
                    .connect(
                        godot::GodotString::from_str("player_timed_out"),
                        Some(*object),
                        godot::GodotString::from_str("_on_net_timed_out"),
                        godot::VariantArray::new(),
                        1,
                    )
                    .unwrap();
                // Connect the new connection signal to the calling gdscript
                let object = &context.to_object();
                _owner
                    .clone()
                    .connect(
                        godot::GodotString::from_str("player_connected"),
                        Some(*object),
                        godot::GodotString::from_str("_on_net_new_connection"),
                        godot::VariantArray::new(),
                        1,
                    )
                    .unwrap();
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
        builder.add_signal(godot::init::Signal {
            name: "player_timed_out",
            args: &[],
        });
        builder.add_signal(godot::init::Signal {
            name: "player_connected",
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
