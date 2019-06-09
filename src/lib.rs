#[macro_use]
extern crate gdnative as godot;
extern crate crossbeam_channel;
extern crate serde_derive;
extern crate laminar;
use crossbeam_channel::{Receiver, Sender};
use laminar::{ErrorKind, Packet, Socket, SocketEvent};
use serde_derive::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::{thread, time};

#[derive(gdnative::NativeClass)]
#[inherit(gdnative::Node)]
struct Laminar {
    client: Option<Client>,
}

struct Client {
    packet_sender: Sender<Packet>,
    _event_receiver: Receiver<SocketEvent>,
    _polling_thread: thread::JoinHandle<Result<(), ErrorKind>>,
}

#[gdnative::methods]
impl Laminar {

    fn _init(_owner: gdnative::Node) -> Self {
        Laminar {
            client: None
        }
    }

    #[export]
    fn _ready(&self, _owner: gdnative::Node) {
        godot_print!("hello, world.")
    }

    #[export]
    fn send(&self, _owner: gdnative::Node, message: godot::GodotString) {
        godot_print!("send packet: {}", message.to_string())
    }

    #[export]
    fn new(&self, _owner: gdnative::Node, address: godot::GodotString) {
        // setup an udp socket and bind it to the client address.
        let (mut socket, packet_sender, event_receiver) = Socket::bind(address.to_string()).unwrap();
        let polling_thread = thread::spawn(move || socket.start_polling());
    }
}

fn init(handle: gdnative::init::InitHandle) {
    handle.add_class::<Laminar>();
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();