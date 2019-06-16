use bincode::{deserialize, serialize};
use crossbeam_channel::{Receiver, Sender};
use datatypes::{PacketData, VariantType, VariantTypes};
use laminar::{ErrorKind, Packet, Socket, SocketEvent};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::{thread, time};

#[derive(Clone)]
pub struct Server {
    pub packet_sender: Sender<Packet>,
    pub event_receiver: Receiver<SocketEvent>,
    pub connections: HashMap<i64, SocketAddr>,
}

struct ShareNode {
    node: godot::Node,
}

unsafe impl Send for ShareNode {}

impl Server {
    pub fn new(port: String) -> Self {
        let address = format!("127.0.0.1:{}", port);
        let listen_address: SocketAddr = match address.to_string().parse() {
            Ok(addr) => addr,
            Err(_) => {
                godot_print!("Laminar: Failed to parse port, defaulting to '8080'");
                "127.0.0.1:8080".to_string().parse().unwrap()
            }
        };
        let (mut socket, packet_sender, event_receiver) = Socket::bind(listen_address).unwrap();
        let polling_thread = thread::spawn(move || socket.start_polling());

        Server {
            packet_sender,
            event_receiver,
            connections: HashMap::new(),
        }
    }

    pub fn send_to_all(&mut self, node_path: String, method: String, variants: VariantTypes) {
        let packet = PacketData {
            node_path,
            method,
            variants,
        };

        let serialized = serialize(&packet);

        match serialized {
            Ok(raw_data) => {
                for addr in self.connections.values() {
                    let packet_sender = self.packet_sender.clone();
                    let raw_data = raw_data.clone();
                    let addr = addr.clone();
                    thread::spawn(move || {
                        &packet_sender
                            .send(Packet::reliable_unordered(addr, raw_data))
                            .unwrap();
                    });
                }
            }
            Err(e) => println!("Some error occurred: {:?}", e),
        }
    }

    pub fn send_to(
        &mut self,
        player_id: i64,
        node_path: String,
        method: String,
        variants: VariantTypes,
    ) {
        let packet = PacketData {
            node_path,
            method,
            variants,
        };

        let serialized = serialize(&packet);

        if let Some(dest_addr) = self.connections.get(&player_id) {
            match serialized {
                Ok(raw_data) => {
                    let packet_sender = self.packet_sender.clone();
                    let raw_data = raw_data.clone();
                    let addr = dest_addr.clone();
                    thread::spawn(move || {
                        &packet_sender
                            .send(Packet::reliable_unordered(addr, raw_data))
                            .unwrap();
                    });
                }
                Err(e) => println!("Some error occurred: {:?}", e),
            }
        }
    }

    pub unsafe fn start_receiving(mut self, owner: godot::Node, context: godot::Node) {
        let mut plugin_node = ShareNode {
            node: owner.clone(),
        };
        let mut context = ShareNode {
            node: context.clone(),
        };

        thread::spawn(move || {
            loop {
                match self.event_receiver.recv() {
                    Ok(SocketEvent::Packet(packet)) => {
                        let received_data: &[u8] = packet.payload();

                        self.connections.insert(1, packet.addr());

                        let data: PacketData = match deserialize(&received_data) {
                            Ok(data) => data,
                            Err(_) => continue,
                        };

                        let target = context
                            .node
                            .get_tree()
                            .unwrap()
                            .get_root()
                            .unwrap()
                            .get_node(godot::NodePath::from_str(&data.node_path));

                        // If the engine cannot find node by our path, drop our data.
                        let target = match target {
                            Some(target) => target,
                            _ => continue,
                        };

                        let target_method = format!("_on_net_{}", &data.method);

                        // Connect the callback signal to the packet's specified destination node.
                        {
                            let object = &target.to_object();

                            plugin_node
                                .node
                                .connect(
                                    godot::GodotString::from_str("recv_data"),
                                    Some(*object),
                                    godot::GodotString::from_str(&target_method), // TODO: use dynamic method names sourced from packet, like "_on_net_foo"
                                    godot::VariantArray::new(),
                                    1,
                                )
                                .unwrap();
                        }

                        // Get the godot variants from the deserialized data variants
                        let mut var_array = godot::VariantArray::new();
                        data.variants
                            .0
                            .iter()
                            .for_each(|var| var_array.push(&var.to_variant()));

                        // Send the variants to the target node and method using godot signals
                        plugin_node.node.emit_signal(
                            godot::GodotString::from_str("recv_data"),
                            &[godot::Variant::from_array(&var_array)],
                        );

                        // Disconnect the callback signal from the packet's specified destination node.
                        {
                            let object = &target.to_object();
                            plugin_node.node.disconnect(
                                godot::GodotString::from_str("recv_data"),
                                Some(*object),
                                godot::GodotString::from_str(&target_method),
                            )
                        }
                        godot_print!("Laminar: Server got packet from {}", packet.addr());
                    }
                    Ok(SocketEvent::Timeout(address)) => {
                        godot_print!("Laminar: Connection to client {} timed out.", address);
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
