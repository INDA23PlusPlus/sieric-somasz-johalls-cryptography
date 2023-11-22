use anyhow::Result;
use libhej;
use ring::digest;
use std::io::prelude::*;
use std::io::stdin;

struct User {
    username: String,
    password: String,
}

struct ObfuscatedFileName {
    name: String,
}

impl From<(User, String)> for ObfuscatedFileName {
    fn from(inp: (User, String)) -> Self {
        let (user, file_name) = inp;
        let mut data = file_name;
        data.push_str(&user.username);
        data.push_str(&user.password);
        ObfuscatedFileName {
            name: format!("{:?}", digest::digest(&digest::SHA512, data.as_bytes())).chars().skip(7).collect(),
        }
    }
}

struct File {
    file_name: String,
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
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(parts[1])?;
                file.write_all(b"From server")?;
            }
            "put" => {
                let mut file = std::fs::File::open(parts[1])?;
                buffer.clear();
                file.read_to_string(&mut buffer)?;

                eprintln!("simulating sending to server: '{}'", buffer);
            }
            _ => {
                dbg_usage();
            }
        }
    }
}
