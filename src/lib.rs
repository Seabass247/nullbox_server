pub mod client;
pub mod datatypes;

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

struct Laminar {
    client: Option<Client>,
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
            callback: false,
        }
    }

    #[export]
    fn _ready(&self, _owner: gdnative::Node) {
        godot_print!("Laminar: plugin ready");
    }

    #[export]
    fn send(&mut self, _owner: gdnative::Node, message: godot::GodotString) {
        match self.client.take() {
            Some(mut client) => {
                client.send(message.to_string());
                godot_print!("Laminar: send packet: {}", message.to_string());
                self.client = Some(client);
            }
            None => {
                godot_print!(
                    "Laminar error: must call function `new_connection` before sending data"
                );
            }
        }
    }
    
    #[export]
    fn send_vars(&mut self, _owner: gdnative::Node, variant: godot::VariantArray) {
        let variant = godot::Variant::from_array(&variant);
        match self.client.take() {
            Some(mut client) => {
                client.send_vars(VariantTypes::from(variant));
                godot_print!("Laminar: send var packet");
                self.client = Some(client);
            }
            None => {
                godot_print!(
                    "Laminar error: must call function `new_connection` before sending data"
                );
            }
        }
    }

    #[export]
    fn init_client(&mut self, _owner: gdnative::Node, address: godot::GodotString, context: godot::Node) {
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

        let client = Client {
            packet_sender,
            _event_receiver: event_receiver,
            server_address,
            uid: None,
        };

        self.client = Some(client);

        match self.client.clone() {
            Some(client) => unsafe {
                client.start_receiving(_owner, context);
                godot_print!(
                    "Laminar: client waiting for connection response from server {}",
                    address.to_string()
                );
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
