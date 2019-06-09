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

struct Laminar {
    client: Option<Client>,
    callback: bool,
}

#[derive(Clone)]
struct Client {
    packet_sender: Sender<Packet>,
    _event_receiver: Receiver<SocketEvent>,
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
        godot_print!("Laminar: plugin initialized!");
        Laminar {
            client: None,
            callback: false,
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
                godot_print!("Laminar: send packet: {}", message.to_string());
                self.client = Some(client);
            }
            None => {
                godot_print!("Laminar error: must call function `new` before sending data");
            }
        }
    }

    #[export]
    fn get_packet(&mut self, mut owner: gdnative::Node) -> godot::ByteArray {
        let mut byte_array = godot::ByteArray::new();
        match self.client.clone() {
            Some(mut client) => {
                match client._event_receiver.recv() {
                    Ok(SocketEvent::Packet(packet)) => {
                        let received_data: &[u8] = packet.payload();
                        let _sent: Vec<u8> = received_data.iter().map(|u| {
                            byte_array.push(*u);
                            *u
                        }).collect();

                        if self.callback {
                            unsafe { owner.emit_signal(godot::GodotString::from_str("recv_data"), &[godot::Variant::from_byte_array(&byte_array)]) };
                        }
                    }
                    Ok(SocketEvent::Timeout(address)) => {
                        godot_print!("Laminar: Connection to server {} timed out.", address);
                    }
                    Ok(_) => {
                        godot_print!("Laminar: got nothing");
                    }
                    Err(e) => {
                        godot_print!("Laminar: Something went wrong when receiving, error: {:?}", e);
                    } 
                }
            }
            None => {
                godot_print!("Laminar error: must call function `new` first");
            }
        }
        byte_array
    }

    #[export]
    unsafe fn test(&mut self, mut _owner: godot::Node) {
        let mut byte_array = godot::ByteArray::new();
        let _sent: Vec<u8> = b"this is test data".iter().map(|u| {
            byte_array.push(*u);
            *u
        }).collect();
        unsafe { _owner.emit_signal(godot::GodotString::from_str("recv_data"), &[godot::Variant::from_byte_array(&byte_array)]) };
    }

    #[export]
    fn new_connection(&mut self, _owner: gdnative::Node, address: godot::GodotString) {
        // setup an udp socket and bind it to the client address.
        let (mut socket, packet_sender, event_receiver) = Socket::bind("127.0.0.1:12346").unwrap();
        let polling_thread = thread::spawn(move || socket.start_polling());

        let server_address: SocketAddr = address.to_string().parse().unwrap();

        let client = Client {
            packet_sender,
            _event_receiver: event_receiver,
            server_address,
        };

        self.client = Some(client);

        godot_print!("Laminar: created new connection to {}", address.to_string());
    }

    #[export]
    unsafe fn set_recv_callback(&mut self, mut _owner: gdnative::Node, target: godot::Node, method: godot::GodotString) {
        //let method = godot::GodotString::from_str("on_data_recv");
        godot_print!("Laminar: set target callback node: {}, method: {}", target.get_name().to_string(), method.to_string());
        let object = &target.to_object();

        _owner
            .connect(
                godot::GodotString::from_str("recv_data"),
                Some(*object),
                method,
                godot::VariantArray::new(),
                1,
            )
            .unwrap();
       
        self.callback = true;
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