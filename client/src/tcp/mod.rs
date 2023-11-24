use libhej::{self, GetResponse};
use libhej::{Get, MessageType, Put};
use serde::Deserialize;
use std::io::prelude::*;
use std::io::stdin;
use std::io::*;
use std::net::TcpStream;

pub fn send_to_server(stream: &TcpStream, id: &String, data: &String) -> std::io::Result<()> {
    let message: MessageType = MessageType::Put(Put {
        id: id.to_owned(),
        data: data.to_owned(),
    });
    serde_json::to_writer(stream, &message)?;
    Ok(())
}
pub fn get_from_server(stream: &TcpStream, id: &String) -> Result<GetResponse> {
    let mesasge: MessageType = MessageType::Get(Get { id: id.to_owned() });
    serde_json::to_writer(stream, &mesasge)?;
    let mut de = serde_json::Deserializer::from_reader(stream);
    let message = match GetResponse::deserialize(&mut de) {
        Ok(v) => v,
        Err(_) => panic!(),
    };
    Ok(message)
}
