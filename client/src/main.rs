use libhej::{Get, MessageType, Put};
use std::io::prelude::*;
use std::io::stdin;
use std::net::TcpStream;

fn main() {
    let mut addr = String::new();
    println!("Type the adress of the server: ");
    stdin().read_line(&mut addr).unwrap();

    let data: MessageType = MessageType::Put(Put {
        id: String::from("123"),
        data: String::from("hej grabs"),
    });
    send_data(addr, data);
}

fn send_data(addr: String, data: MessageType) -> std::io::Result<()> {
    let mut stream = TcpStream::connect(addr)?;
    match &data {
        MessageType::Get(_) => panic!("Wrong MessageType"),
        MessageType::Put(_) => {
            serde_json::to_writer(&stream, &data);
            Ok(())
        }
    }
}
