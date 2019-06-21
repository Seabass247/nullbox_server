use bincode::{deserialize, serialize};
use crossbeam_channel::{Receiver, Sender};
use datatypes::{PacketData, VariantType, VariantTypes, MetaMessage};
use laminar::{ErrorKind, Packet, Socket, SocketEvent};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::{thread, time};

#[derive(Clone)]
pub struct Server {
    pub packet_sender: Sender<Packet>,
    pub event_receiver: Receiver<SocketEvent>,
    pub player_conns: HashMap<SocketAddr, i64>,
    pub new_conn_ch: (Sender<(SocketAddr, i64)>, Receiver<(SocketAddr, i64)>),
    pub timeout_conn_ch: (Sender<(SocketAddr, i64)>, Receiver<(SocketAddr, i64)>),
}

struct ShareNode {
    node: godot::Node,
}

unsafe impl Send for ShareNode {}

impl Server {
    pub fn new(owner: godot::Node, port: String) -> Self {
        let address = format!("127.0.0.1:{}", port);
        let listen_address: SocketAddr = match address.to_string().parse() {
            Ok(addr) => addr,
            Err(_) => {
                godot_print!("Laminar: Failed to parse port, defaulting to '8080'");
                "127.0.0.1:8080".to_string().parse().unwrap()
            }
        };
        let (mut socket, packet_sender, event_receiver) = Socket::bind(listen_address).unwrap();
        let event = event_receiver.clone();
        thread::spawn(move || {
            socket.start_polling();
        });

        let (tx_new_conn, rx_new_conn): (Sender<(SocketAddr, i64)>, Receiver<(SocketAddr, i64)>) = crossbeam_channel::unbounded();
        let (tx_timeout_conn, rx_timeout_conn): (Sender<(SocketAddr, i64)>, Receiver<(SocketAddr, i64)>) = crossbeam_channel::unbounded();

        let server = Server {
            packet_sender,
            event_receiver,
            player_conns: HashMap::new(),
            new_conn_ch: (tx_new_conn, rx_new_conn),
            timeout_conn_ch: (tx_timeout_conn, rx_timeout_conn),
        };
        
        server
    }

    pub fn send_to_all(&mut self, node_path: String, method: String, variants: VariantTypes) {
        self.update_player_conns();

        let packet = PacketData {
            node_path,
            method,
            variants,
        };

        let serialized = serialize(&packet);

        match serialized {
            Ok(raw_data) => {
                for addr in self.player_conns.keys() {
                    let packet_sender = self.packet_sender.clone();
                    let raw_data = raw_data.clone();
                    let addr = addr.clone();
                    &packet_sender
                        .send(Packet::reliable_unordered(addr, raw_data))
                        .unwrap();

                }
            }
            Err(e) => println!("Some error occurred: {:?}", e),
        }
    }

    fn update_player_conns(&mut self) {
        // For every timed out player connection sent from the recv thread, remove the player from this thread's player dict.
        while let Ok(tup) = self.timeout_conn_ch.1.try_recv() {
            let addr = tup.0;
            self.player_conns.remove(&addr);
        }
        // For every new player connection sent from the recv thread, add the player to this thread's player dict.
        while let Ok(tup) = self.new_conn_ch.1.try_recv() {
            let addr = tup.0;
            let id = tup.1;
            self.player_conns.insert(addr, id);
        }
    }

    pub fn send_to(
        &mut self,
        player_id: i64,
        node_path: String,
        method: String,
        variants: VariantTypes,
    ) {
        let conns = self.update_player_conns();

        let packet = PacketData {
            node_path,
            method,
            variants,
        };

        let serialized = serialize(&packet);

        // Send packet to the address that's associate with id 'player_id'
        for (addr, id) in &self.player_conns {
            if id == &player_id {
                match &serialized {
                    Ok(raw_data) => {
                        let packet_sender = self.packet_sender.clone();
                        let raw_data = raw_data.clone();
                        let addr = addr.clone();
                        &packet_sender
                            .send(Packet::reliable_unordered(addr, raw_data))
                            .unwrap();

                    }
                    Err(_) => println!("Some error occurred serializing"),
                }           
            }
        }
    }

    pub unsafe fn start_receiving(&self, owner: godot::Node) {
        let mut plugin_node = ShareNode {
            node: owner.clone(),
        };
        let tx_player = self.new_conn_ch.0.clone();
        let tx_timed_out_player = self.timeout_conn_ch.0.clone();
        let event_receiver = self.event_receiver.clone();

        thread::spawn(move || {
            let mut unique_client_id: i64 = 0;
            let mut current_id: i64 = 0;
            let mut player_id_dict: HashMap<SocketAddr, i64> = HashMap::new();

            loop {
                //godot_print!("Current player dict={:?}", player_id_dict);
                match event_receiver.recv() {
                    Ok(SocketEvent::Packet(packet)) => {
                        let received_data: &[u8] = packet.payload();
                        
                        let data: PacketData = match deserialize(&received_data) {
                            Ok(data) => data,
                            // Handle non-in-game received data
                            Err(_) => {
                                match deserialize::<MetaMessage>(&received_data) {
                                    Ok(data) => {
                                        match data {
                                            MetaMessage::Ack => {},
                                            MetaMessage::Heartbeat => {},
                                        }
                                        continue;
                                    },
                                    Err(_) => {
                                        continue
                                    }
                                }
                            },
                        };
                        
                        // If this packet address a known player id associated with it, set the current id.
                        if let Some(id) = player_id_dict.get(&packet.addr()) {
                            current_id = *id;
                        }
                        
                        // No known id for this packet address, let's get a new id and set current id
                        if player_id_dict.get(&packet.addr()).is_none() {
                            godot_print!("Laminar Server: New connection from {}", &packet.addr());
                            unique_client_id += 1;
                            current_id = unique_client_id;
                            player_id_dict.insert(packet.addr(), current_id);
                            tx_player.send((packet.addr(), current_id));

                            // Pass the new connection event through a godot signal so we can handle it in gdscript.
                            plugin_node.node.emit_signal(
                                godot::GodotString::from_str("player_connected"),
                                &[godot::Variant::from_i64(unique_client_id)],
                            );
                        }

                        let target = plugin_node
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
                                    godot::GodotString::from_str(&target_method),
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
                            &[godot::Variant::from_i64(current_id), godot::Variant::from_array(&var_array)],
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
                        //godot_print!("Laminar: Server got packet from {}", packet.addr());
                    }
                    Ok(SocketEvent::Timeout(address)) => {
                        godot_print!("Laminar: Connection to client {} timed out.", address);

                        let timed_out_player_id = player_id_dict.remove(&address).unwrap();
    
                        // Pass the timed out connection event through a godot signal so we can handle it in gdscript
                        plugin_node.node.emit_signal(
                            godot::GodotString::from_str("player_timed_out"),
                            &[godot::Variant::from_i64(timed_out_player_id)],
                        );
                        
                        // Share the timed out player with the main lib thread so dont send data in future calls
                        tx_timed_out_player.send((address, timed_out_player_id));
                    }
                    Ok(_) => {
                    }
                    Err(e) => {
                        godot_print!(
                            "Laminar: #RECV ERROR#: {:?}",
                            e
                        );
                    }
                }
            }
        });
    }
}
