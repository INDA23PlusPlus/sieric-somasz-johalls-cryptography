use std::{collections::HashMap, net::TcpListener, io::Read};
use libhej::{self, MessageType, GetResponse};
use serde::de::Deserialize;

struct State {
    map: HashMap<String, String>,
}

impl State {
    pub fn new() -> Self {
        return State {
            map: HashMap::new(),
        };
    }
}

fn put(state: &mut State, fileid: String, file: String) {
    state.map.insert(fileid, file);
}

fn get(state: &State, fileid: String) -> GetResponse {
    if let Some(v) = state.map.get(&fileid) {
        return GetResponse {
            data: Some(v.clone()),
        };
    }
    return GetResponse {
        data: None,
    };
}

fn main() -> Result<(), std::io::Error> {
    let mut state = State::new();

    let listener = TcpListener::bind("0.0.0.0:31337")?;

    loop {
        let (stream, addr) = listener.accept()?;
        println!("Connected to {}", addr);

        let mut de = serde_json::Deserializer::from_reader(&stream);

        loop {
            let message = match MessageType::deserialize(&mut de) {
                Ok(v) => v,
                Err(_) => break,
            };
            match message {
                MessageType::Put(a) => {
                    put(&mut state, a.id, a.data);
                },
                MessageType::Get(a) => {
                    match serde_json::to_writer(&stream, &get(&mut state, a.id)) {
                        Ok(_) => (),
                        Err(_) => break,
                    };
                },
            };
        }
    }
}
