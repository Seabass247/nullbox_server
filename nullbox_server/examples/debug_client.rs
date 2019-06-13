use std::io::stdin;

use laminar::{ErrorKind, Packet, Socket, SocketEvent};
use std::thread;

const SERVER: &str = "127.0.0.1:12345";

fn main() {
    let addr = "127.0.0.1:12352";
    let (mut socket, packet_sender, event_receiver) =
        Socket::bind(addr).expect("Cannot bind to address");
    println!("Connected on {}", addr);
    let _thread = thread::spawn(move || socket.start_polling());

    let server = SERVER.parse().unwrap();

    println!("Type a message and press Enter to send. Send `Bye!` to quit.");

    let stdin = stdin();
    let mut s_buffer = String::new();

    loop {
        s_buffer.clear();
        stdin.read_line(&mut s_buffer).expect("Failed to read line");

        let line = s_buffer.replace(|x| x == '\n' || x == '\r', "");

        packet_sender
            .send(Packet::reliable_unordered(
                server,
                line.clone().into_bytes(),
            ))
            .unwrap();
    }
}
