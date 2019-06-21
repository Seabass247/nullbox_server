use bincode::{deserialize, serialize};
use crossbeam_channel::{Receiver, Sender};
use datatypes::{PacketData, VariantType, VariantTypes, MetaMessage};
use laminar::{ErrorKind, Packet, Socket, SocketEvent};
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
    pub recv_sleep: (Sender<bool>, Receiver<bool>),
    pub current_root: (Sender<String>, Receiver<String>),
}

struct ShareNode {
    node: godot::Node,
}

unsafe impl Send for ShareNode {}

impl Client {
    pub fn send_vars(&mut self, node_path: String, method: String, data_types: VariantTypes) {
        let packet = PacketData {
            node_path,
            method,
            variants: data_types,
        };

        let serialized = serialize(&packet);
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

    pub fn send_sync(&mut self, message: MetaMessage) {
        let packet = MetaMessage::Heartbeat;

        let serialized = serialize(&packet);
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

    pub unsafe fn start_receiving(self, owner: godot::Node) {
        let mut plugin_node = ShareNode {
            node: owner.clone(),
        };
        let rx_sleep = self.recv_sleep.1.clone();
        let rx_root = self.current_root.1.clone();

        thread::spawn(move || {
            let mut recv_sleep = false;
            let mut current_root = String::new();

            loop {
                match self._event_receiver.recv() {
                    Ok(SocketEvent::Packet(packet)) => {
                        // Ignore packets if client is supposed to sleep.
                        if recv_sleep {
                            match rx_sleep.try_recv() {
                                Ok(sleep) => {
                                    recv_sleep = sleep;
                                }
                                _ => {}
                            }
                            continue;
                        }
                        
                        // Update our current root from godot
                        match rx_root.try_recv() {
                            Ok(root) => {
                                current_root = root;
                            }
                            _ => {}
                        }
                        
                        let received_data: &[u8] = packet.payload();

                        let data: PacketData = match deserialize(&received_data) {
                            Ok(data) => data,
                            Err(_) => continue,
                        };

                        // Don't process data if the data's destination node path is outside our current scene root.
                        let packet_root = data.node_path.split("/").collect::<Vec<&str>>().get(2).unwrap().to_string();
                        let dest_root = current_root.split("/").collect::<Vec<&str>>().get(2).unwrap().to_string();
                        if dest_root != packet_root {
                            continue;
                        }

                        let target = plugin_node
                            .node
                            .get_tree()
                            .unwrap()
                            .get_root()
                            .unwrap()
                            .get_node(godot::NodePath::from_str(&data.node_path));

                        // If the engine cannot find node by our path, skip.
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
                        godot_print!("Laminar: Client got packet from {}", packet.addr());
                        
                        // Update recv_sleep to what we get from godot
                        match rx_sleep.try_recv() {
                            Ok(sleep) => {
                                recv_sleep = sleep;
                            }
                            _ => {}
                        }
                    }
                    Ok(SocketEvent::Timeout(address)) => {
                        godot_print!("Laminar: Connection to server {} timed out.", address);
                        // TODO: add timeout signal, so the client can handle it
                    }
                    Ok(_) => {
                        godot_print!("Laminar: Got nothing");
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
