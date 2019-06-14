#[macro_use]
extern crate gdnative as godot;
extern crate bincode;
extern crate crossbeam_channel;
extern crate laminar;
extern crate nullbox_core as nullbox;
extern crate serde_derive;

use bincode::{deserialize, serialize};
use crossbeam_channel::{Receiver, Sender};
use laminar::{ErrorKind, Packet, Socket, SocketEvent};
use nullbox::DataType;
use serde_derive::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::{thread, time};

struct Laminar {
    client: Option<Client>,
    callback: bool,
}

#[derive(Clone)]
struct Client {
    packet_sender: Sender<Packet>,
    _event_receiver: Receiver<SocketEvent>,
    server_address: SocketAddr,
    uid: Option<String>,
}

struct ShareNode {
    node: godot::Node,
}

unsafe impl Send for ShareNode {}

impl Client {
    pub fn send(&mut self, msg: String) {
        let raw_data = msg.as_bytes().to_vec();
        let packet_sender = self.packet_sender.clone();
        let addr = self.server_address;
        thread::spawn(move || {
            &packet_sender
                .send(Packet::reliable_unordered(addr, raw_data))
                .unwrap();
        });
    }

    pub unsafe fn start_receiving(self, owner: godot::Node, context: godot::Node) {
        let mut byte_array = godot::ByteArray::new();
        let mut plugin_node = ShareNode {
            node: owner.clone(),
        };
        let mut context = ShareNode {
            node: context.clone(),
        };

        thread::spawn(move || {
            loop {
                match self._event_receiver.recv() {
                    Ok(SocketEvent::Packet(packet)) => {
                        let received_data: &[u8] = packet.payload();
                        let received_data_str = match std::str::from_utf8(received_data) {
                            Ok(data) => data,
                            _ => continue,
                        };

                        // Stop processing packet if there is no valid id separator ("#").
                        let id_and_data: Vec<&str> = received_data_str.split("#").collect();
                        if id_and_data.len() <= 1 {
                            continue;
                        }

                        // Stop processing packet if there is no valid path separator (">").
                        let id_path_and_data: Vec<&str> = received_data_str.split(">").collect();
                        if id_path_and_data.len() <= 1 {
                            continue;
                        }

                        let data_str_parts: Vec<&str> =
                            received_data_str.split(|c| c == '#' || c == '>').collect();

                        let recv_uid = data_str_parts[0];
                        let node_path = data_str_parts[1];
                        let data_body = data_str_parts[2];

                        // Fill the data array with fields and their subfield labels and values
                        // Example: "foo,4,3,2;bar,6,3,7" => [ ["foo","4","3","2"], ["bar","6","3","7"] ]
                        let mut data_array = godot::VariantArray::new();
                        data_body
                            .split(";")
                            .for_each(|field| if !field.is_empty() {
                                let field_split: Vec<&str> = field.split(",").collect();
                                let mut subfield_array = godot::StringArray::new();
                                subfield_array.push(&godot::GodotString::from_str(field_split[0]));
                                subfield_array.push(&godot::GodotString::from_str(field_split[1]));
                                data_array.push(&godot::Variant::from_string_array(&subfield_array));
                            });

                        // If this client has an assigned unique network id, decide whether the recv'd id matches it.
                        if let Some(id) = &self.uid {
                            match id.as_str() {
                                // Anyone can read 0 id packets. Keep processing...
                                "0" => {}
                                // Covers all ids not 0; is it our client's id?  If not, stop processing.
                                _ => {
                                    if id != recv_uid {
                                        continue;
                                    }
                                }
                            }
                        }

                        let _sent: Vec<u8> = received_data
                            .iter()
                            .map(|u| {
                                byte_array.push(*u);
                                *u
                            })
                            .collect();

                        let target = context
                            .node
                            .get_tree()
                            .unwrap()
                            .get_root()
                            .unwrap()
                            .get_node(godot::NodePath::from_str(node_path));

                        // If the engine cannot find node by our path, drop our data.
                        let target = match target {
                            Some(target) => target,
                            _ => continue,
                        };

                        // Connect the callback signal to the packet's specified destination node.
                        {
                            let object = &target.to_object();

                            plugin_node
                                .node
                                .connect(
                                    godot::GodotString::from_str("recv_data"),
                                    Some(*object),
                                    godot::GodotString::from_str("on_network_received"), // TODO: use dynamic method names sourced from packet, like "_on_net_foo"
                                    godot::VariantArray::new(),
                                    1,
                                )
                                .unwrap();
                        }

                        // Use godot signal to send data to the target node's callback function.
                        plugin_node.node.emit_signal(
                            godot::GodotString::from_str("recv_data"),
                            &[godot::Variant::from_array(&data_array)],
                        );

                        // Disconnect the callback signal from the packet's specified destination node.
                        {
                            let object = &target.to_object();
                            plugin_node.node.disconnect(
                                godot::GodotString::from_str("recv_data"),
                                Some(*object),
                                godot::GodotString::from_str("on_network_received"),
                            )
                        }
                        byte_array = godot::ByteArray::new();
                        godot_print!("Laminar: Got packet from {}", packet.addr());
                    }
                    Ok(SocketEvent::Timeout(address)) => {
                        godot_print!("Laminar: Connection to server {} timed out.", address);
                    }
                    Ok(_) => {
                        godot_print!("Laminar: got nothing");
                    }
                    Err(e) => {
                        godot_print!(
                            "Laminar: Something went wrong when receiving, error: {:?}",
                            e
                        );
                    }
                }
            }
        });
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
    fn start_receiving(&mut self, mut owner: gdnative::Node, context: godot::Node) {
        match self.client.clone() {
            Some(client) => unsafe {
                client.start_receiving(owner, context);
                godot_print!(
                    "Laminar: listening for incoming packets... will forward them to recv callback"
                );
            },
            None => {
                godot_print!("Laminar error: must call function `new_connection` first");
            }
        }
    }

    #[export]
    fn get_packet(&mut self, mut owner: gdnative::Node) -> godot::ByteArray {
        let mut byte_array = godot::ByteArray::new();
        match self.client.clone() {
            Some(mut client) => match client._event_receiver.recv() {
                Ok(SocketEvent::Packet(packet)) => {
                    let received_data: &[u8] = packet.payload();
                    let _sent: Vec<u8> = received_data
                        .iter()
                        .map(|u| {
                            byte_array.push(*u);
                            *u
                        })
                        .collect();

                    if self.callback {
                        unsafe {
                            owner.emit_signal(
                                godot::GodotString::from_str("recv_data"),
                                &[godot::Variant::from_byte_array(&byte_array)],
                            )
                        };
                    }
                }
                Ok(SocketEvent::Timeout(address)) => {
                    godot_print!("Laminar: Connection to server {} timed out.", address);
                }
                Ok(_) => {
                    godot_print!("Laminar: got nothing");
                }
                Err(e) => {
                    godot_print!(
                        "Laminar: Something went wrong when receiving, error: {:?}",
                        e
                    );
                }
            },
            None => {
                godot_print!("Laminar error: must call function `new_connection` first");
            }
        }
        byte_array
    }

    #[export]
    unsafe fn test(&mut self, mut _owner: godot::Node) {
        let mut byte_array = godot::ByteArray::new();
        let _sent: Vec<u8> = b"this is test data"
            .iter()
            .map(|u| {
                byte_array.push(*u);
                *u
            })
            .collect();
        unsafe {
            _owner.emit_signal(
                godot::GodotString::from_str("recv_data"),
                &[godot::Variant::from_byte_array(&byte_array)],
            )
        };
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
            uid: None,
        };

        self.client = Some(client);

        godot_print!("Laminar: created new connection to {}", address.to_string());
    }

    #[export]
    unsafe fn set_recv_callback(&mut self, mut _owner: gdnative::Node, target: godot::Node) {
        //let method = godot::GodotString::from_str("on_data_recv");
        godot_print!(
            "Laminar: set recv callback to node: {}",
            target.get_name().to_string()
        );
        let object = &target.to_object();

        _owner
            .connect(
                godot::GodotString::from_str("recv_data"),
                Some(*object),
                godot::GodotString::from_str("on_network_received"),
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
