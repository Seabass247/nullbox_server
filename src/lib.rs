#[macro_use]
extern crate gdnative as godot;
extern crate crossbeam_channel;
extern crate serde_derive;
extern crate laminar;
extern crate bincode;
extern crate nullbox_core as nullbox;

use bincode::{deserialize, serialize};
use crossbeam_channel::{Receiver, Sender};
use laminar::{ErrorKind, Packet, Socket, SocketEvent};
use serde_derive::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::{thread, time};
use nullbox::DataType;

#[derive(gdnative::NativeClass)]
#[inherit(gdnative::Node)]
struct Laminar {
    client: Option<Client>,
}

struct Client {
    packet_sender: Sender<Packet>,
    _event_receiver: Receiver<SocketEvent>,
    _polling_thread: thread::JoinHandle<Result<(), ErrorKind>>,
    server_address: SocketAddr,
}

impl Client {
    pub fn send(&mut self, data_type: DataType) {
        let serialized = serialize(&data_type);

        match serialized {
            Ok(raw_data) => {
                self.packet_sender
                    .send(Packet::reliable_unordered(self.server_address, raw_data))
                    .unwrap();
            }
            Err(e) => println!("Some error occurred: {:?}", e),
        }
    }
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
    fn send(&mut self, _owner: gdnative::Node, message: godot::GodotString) {
        match self.client.take() {
            Some(mut client) => {
                client.send(DataType::ASCII {
                    string: message.to_string()
                });
                godot_print!("send packet: {}", message.to_string());
            }
            None => {
                godot_print!("Laminar error: must call function `new` before sending data");
            }
        }
    }

    #[export]
    fn new(&mut self, _owner: gdnative::Node, address: godot::GodotString) {
        // setup an udp socket and bind it to the client address.
        let (mut socket, packet_sender, event_receiver) = Socket::bind("127.0.0.1:12346").unwrap();
        let polling_thread = thread::spawn(move || socket.start_polling());

        let server_address: SocketAddr = address.to_string().parse().unwrap();

        let client = Client {
            packet_sender,
            _event_receiver: event_receiver,
            _polling_thread: polling_thread,
            server_address,
        };

        self.client = Some(client);
    }
}

fn init(handle: gdnative::init::InitHandle) {
    handle.add_class::<Laminar>();
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();