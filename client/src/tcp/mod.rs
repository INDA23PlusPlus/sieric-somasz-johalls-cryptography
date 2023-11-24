use libhej::{self, GetResponse};
use libhej::{Get, MessageType, Put};
use std::io::prelude::*;
use std::io::stdin;
use std::io::*;
use std::net::TcpStream;

pub struct EncryptedFile {
    pub name: ObfuscatedFileName,
    pub contents: String,
}

pub struct ObfuscatedFileName {
    pub name: String,
}

// impl Default for ObfuscatedFileName {
//     fn default() -> Self {
//         ObfuscatedFileName {
//             name: ring::rand::generate::<[u8; 32]>(&ring::rand::SystemRandom::new())
//                 .unwrap()
//                 .expose(),
//         }
//     }
// }

pub fn send_to_server(stream: &TcpStream, data: &String, id: &String) -> std::io::Result<()> {
    let message: MessageType = MessageType::Put(Put {
        id: id.to_owned(),
        data: data.to_owned(),
    });
    serde_json::to_writer(stream, &message)?;
    Ok(())
}
pub fn get_from_server(stream: &TcpStream, id: String) -> Result<GetResponse> {
    let mesasge: MessageType = MessageType::Get(Get { id: id.to_owned() });
    serde_json::to_writer(stream, &mesasge)?;
    Ok(serde_json::from_reader(stream)?)
}
