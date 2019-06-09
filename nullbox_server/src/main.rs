mod server;
use bincode::{deserialize, serialize};
use crossbeam_channel::{Receiver, Sender};
use laminar::{ErrorKind, Packet, Socket, SocketEvent};
use serde_derive::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::{thread, time};
use crate::server::Server;

fn main() {
    println!("NullBox Server. Listening for connections on {}", server::server_address());

    let mut server = Server::new();
    // set up or `Server` that will receive the messages we send with the `Client`
    let handle = thread::spawn(move || loop {
        server.receive();
    });

    loop {

    }
}
