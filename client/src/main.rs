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


use anyhow::Result;
use libhej;
use ring::digest;
use std::io::*;

struct User {
    username: String,
    password: String,
}

struct ObfuscatedFileName {
    name: [u8; 32],
}

impl Default for ObfuscatedFileName {
    fn default() -> Self {
        ObfuscatedFileName {
            name: ring::rand::generate::<[u8; 32]>(&ring::rand::SystemRandom::new())
                .unwrap()
                .expose(),
        }
    }
}

struct File {
    file_name: String,
    obf_file_name: ObfuscatedFileName,
    last_hash: String,
}



fn main() -> Result<()> {
    let mut buffer = String::new();
    loop {
        buffer.clear();
        stdin().read_line(&mut buffer)?;

        let parts = buffer
            .trim()
            .split(" ")
            .map(|s| s.trim())
            .filter(|s| s.len() > 0)
            .collect::<Vec<_>>();

        let dbg_usage = || {
            eprintln!("Usage: 'get [filename]' or 'put [filename]'");
        };

        if parts.len() != 2 {
            dbg_usage();
            continue;
        }

        match parts[0] {
            "get" => {
                let mut file = match std::fs::File::open(parts[1]) {
                    Ok(v) => v,
                    Err(e1) => match std::fs::File::create(parts[1]) {
                        Ok(v) => v,
                        Err(e2) => {
                            eprintln!("error opening file: '{:?}' tried to open '{}'", e1, parts[1]);
                            eprintln!("error creating file: '{:?}' tried to creat '{}'", e2, parts[1]);
                            continue;
                        }
                    },
                };

                file.write_all()
            }
            "put" => {
                let mut file = match std::fs::File::open(parts[1]) {
                    Err(e) => {
                        eprintln!("error opening file: '{:?}' tried to open '{}'", e, parts[1]);
                        continue;
                    }
                    Ok(v) => v,
                };
                buffer.clear();
                file.read_to_string(&mut buffer);

                eprintln!("simulating sending to server: '{}'", buffer);
            }
            _ => {
                dbg_usage();
            }
        }
    }
}
