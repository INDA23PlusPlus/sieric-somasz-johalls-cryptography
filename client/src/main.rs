use anyhow::Result;
use libhej;
use ring::digest;
use std::io::prelude::*;
use std::io::stdin;
use std::str::from_utf8_unchecked;

struct User {
    username: String,
    password: String,
}

struct ObfuscatedFileName {
    name: String,
}

fn get_hash(s: &str) -> String {
    format!("{:?}", digest::digest(&digest::SHA512, s.as_bytes()))
                .chars()
                .skip(7)
                .collect()
}

impl From<(User, String)> for ObfuscatedFileName {
    fn from(inp: (User, String)) -> Self {
        let (user, file_name) = inp;
        let mut data = file_name;
        data.push_str(&user.username);
        data.push_str(&user.password);
        ObfuscatedFileName {
            name: get_hash(data.as_str()),
        }
    }
}

struct File {
    file_name: String,
    last_hash: String,
}

fn main() -> Result<()> {
    let mut buffer = String::new();

    let mut files = std::collections::HashMap::<String, String>::new();
    if let Ok(mut file) = std::fs::File::open("opened_files.cache") {
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        for line in buf.lines() {
            let line = line?;
            let mut parts = line.split_ascii_whitespace();
            let name = parts.next().unwrap();
            let hash = parts.next().unwrap();
            files.insert(name.to_owned(), hash.to_owned());
        }
    }

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

                let from_server = "From server".to_owned();

                file.write_all(from_server.as_bytes())?;
                if let Some(r) = files.get_mut(parts[1]) {
                    if *r != get_hash(from_server.as_str()) {
                        println!("FILE CONTENTS DO NOT MATCH");
                        println!("Local hash: {}", *r);
                        println!("Hash of file from server {}", get_hash(from_server.as_str()));
                    }
                } else {
                    println!("WARNING, no local hash of file '{}'", parts[1]);
                }

            }
            "put" => {
                let mut file = std::fs::File::open(parts[1])?;
                let mut buffer = String::new();
                file.read_to_string(&mut buffer)?;
                eprintln!("simulating sending to server: '{}'", buffer);
                files.insert(parts[1].to_owned(), get_hash(buffer.as_str()));
                let mut outp = String::new();
                for (file, hash) in &files {
                    outp.push_str(&file);
                    outp.push(' ');
                    outp.push_str(&hash);
                    outp.push('\n');
                }
                let mut files_cache = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open("opened_files.cache")?;
                files_cache.write_all(outp.as_bytes())?;
            }
            _ => {
                dbg_usage();
            }
        }
    }
}
