pub mod client;

#[macro_use]
extern crate gdnative as godot;
#[macro_use]
use bincode::{deserialize, serialize};
use crossbeam_channel::{Receiver, Sender};
use laminar::{ErrorKind, Packet, Socket, SocketEvent};
use serde_derive::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::{thread, time};
use client::player::PlayerClient;

/// The socket address of where the server is located.
const SERVER_ADDR: &'static str = "127.0.0.1:12345";
// The client address from where the data is sent.
const CLIENT_ADDR: &'static str = "127.0.0.1:12346";

#[derive(godot::NativeClass)]
#[inherit(godot::Node)]
struct Game {
    client: Option<Box<Client>>,
    player: Option<Box<godot::KinematicBody>>,
}

#[godot::methods]
impl Game {

    fn _init(_owner: godot::Node) -> Self {
        Game {
            client: None,
            player: None,
        }
    }

    #[export]
    fn _ready(&mut self, owner: godot::Node) {
        godot_print!("Rust Game lib ready");
        
        // Connect signal "moved" to our callback function
        unsafe {
            let mut source = owner.get_node(godot::NodePath::from_str("Player")).unwrap();
            let target_obj = &owner.to_object();

            source
                .connect(
                    godot::GodotString::from_str("moved"),
                    Some(*target_obj),
                    godot::GodotString::from_str("moved_callback"),
                    godot::VariantArray::new(),
                    1,
                )
                .unwrap();
        }
        
        // Expose our UDP client to this "class"
        let mut client = Client::new();
        self.client = Some(Box::new(client));
        
        // Expose this clients Player node to the class
        unsafe {
            let player_node = owner.get_node(godot::NodePath::from_str("Player")).unwrap();
            let player = player_node.cast::<godot::KinematicBody>().unwrap();
            self.player = Some(Box::new(player));
        }
        
    }

    #[export]
    fn _process(&mut self, _owner: godot::Node, delta: f32) {
        
    }

    #[export]
    fn moved_callback(&mut self, mut _owner: godot::Node) {
        let mut client = self.client.as_mut().unwrap();
        let player = &self.player.as_ref().unwrap();
        unsafe {
            let pos = player.get_translation();
            client.send(DataType::Coords {
                x: pos.x,
                y: pos.y,
                z: pos.z,
            });
            godot_print!("Sent postion: {:?}", pos);
        }
    }
}

fn init(handle: godot::init::InitHandle) {
    handle.add_class::<Game>();
    handle.add_class::<PlayerClient>();
}

fn client_address() -> SocketAddr {
    CLIENT_ADDR.parse().unwrap()
}

fn server_address() -> SocketAddr {
    SERVER_ADDR.parse().unwrap()
}

#[derive(Serialize, Deserialize)]
enum DataType {
    Coords {
        x: f32,
        y: f32,
        z: f32,
    },
    Name {
        string: String,
    },
}

struct Client {
    packet_sender: Sender<Packet>,
    _event_receiver: Receiver<SocketEvent>,
    _polling_thread: thread::JoinHandle<Result<(), ErrorKind>>,
}

impl Client {
    pub fn new() -> Self {
        // setup an udp socket and bind it to the client address.
        let (mut socket, packet_sender, event_receiver) = Socket::bind(client_address()).unwrap();
        let polling_thread = thread::spawn(move || socket.start_polling());

        Client {
            packet_sender,
            _event_receiver: event_receiver,
            _polling_thread: polling_thread,
        }
    }

    pub fn send(&mut self, data_type: DataType) {
        let serialized = serialize(&data_type);

        match serialized {
            Ok(raw_data) => {
                self.packet_sender
                    .send(Packet::reliable_unordered(server_address(), raw_data))
                    .unwrap();
            }
            Err(e) => println!("Client: Some error occurred: {:?}", e),
        }
    }
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();