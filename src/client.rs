use bincode::{deserialize, serialize};
use crossbeam_channel::{Receiver, Sender};
use laminar::{ErrorKind, Packet, Socket, SocketEvent};
use datatypes::{VariantType, VariantTypes};
use serde_derive::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::net::{IpAddr, Ipv4Addr};
use std::{thread, time};

#[derive(Clone)]
pub struct Client {
    pub packet_sender: Sender<Packet>,
    pub _event_receiver: Receiver<SocketEvent>,
    pub server_address: SocketAddr,
    pub uid: Option<String>,
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

    pub fn send_vars(&mut self, data_types: VariantTypes) {
        let serialized = serialize(&data_types);
        let packet_sender = self.packet_sender.clone();
        let addr = self.server_address;

        match serialized {
            Ok(raw_data) => {
                thread::spawn(move || {
                    &packet_sender
                        .send(Packet::reliable_unordered(addr, raw_data))
                        .unwrap();
                });
            }
            Err(e) => println!("Some error occurred: {:?}", e),
        }        
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
                        // Example: "foo=4,3,2;bar=6,3,7" => [ ["foo","4","3","2"], ["bar","6","3","7"] ]
                        let mut data_array = godot::VariantArray::new();
                        data_body.split(";").for_each(|field| {
                            if !field.is_empty() {
                                let mut subfield_array = godot::StringArray::new();
                                field.split(|c| c == '=' || c == ',').for_each(|subfield| {
                                    if !subfield.is_empty() {
                                        subfield_array
                                            .push(&godot::GodotString::from_str(subfield));
                                    }
                                });
                                data_array
                                    .push(&godot::Variant::from_string_array(&subfield_array));
                            }
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

                        // Use godot signal to send the data to the target node's callback function.
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
                        //godot_print!("Laminar: Got packet from {}", packet.addr());
                    }
                    Ok(SocketEvent::Timeout(address)) => {
                        godot_print!("Laminar: Connection to server {} timed out.", address);
                        //break;
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
