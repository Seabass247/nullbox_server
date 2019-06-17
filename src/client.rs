use bincode::{deserialize, serialize};
use crossbeam_channel::{Receiver, Sender};
use datatypes::{PacketData, VariantType, VariantTypes};
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
    pub recv_sleep: (Sender<std::time::Duration>, Receiver<std::time::Duration>),
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

    pub unsafe fn start_receiving(self, owner: godot::Node) {
        let mut plugin_node = ShareNode {
            node: owner.clone(),
        };
        let rx_sleep = self.recv_sleep.1.clone();

        thread::spawn(move || {
            loop {
                match self._event_receiver.recv() {
                    Ok(SocketEvent::Packet(packet)) => {
                        while let Ok(time) = rx_sleep.try_recv() {
                            godot_print!("LAMINAR SLEEP");
                            std::thread::sleep(time);
                            continue;
                        }
                        let received_data: &[u8] = packet.payload();

                        let data: PacketData = match deserialize(&received_data) {
                            Ok(data) => data,
                            Err(_) => continue,
                        };

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
                    }
                    Ok(SocketEvent::Timeout(address)) => {
                        godot_print!("Laminar: Connection to server {} timed out.", address);
                        //break;
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
