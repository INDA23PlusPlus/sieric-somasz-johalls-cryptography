mod tcp;

use anyhow::Result;
use base64::Engine;
use libhej;
use ring::aead::Aad;
use ring::aead::BoundKey;
use ring::aead::Nonce;
use ring::aead::NonceSequence;
use ring::aead::OpeningKey;
use ring::aead::SealingKey;
use ring::aead::UnboundKey;
use ring::rand::SecureRandom;
use std::collections::HashMap;
use std::io::prelude::*;
use std::io::stdin;
use std::io::stdout;
use std::net::TcpStream;
use std::num::NonZeroU32;

use crate::tcp::get_from_server;
use crate::tcp::send_to_server;

#[derive(Clone)]
struct User {
    username: String,
    password: String,
}
static PBKDF2_ALG: ring::pbkdf2::Algorithm = ring::pbkdf2::PBKDF2_HMAC_SHA256;
const ENCRYPTION_KEY_LEN: usize = ring::digest::SHA256_OUTPUT_LEN;
type EncryptionKey = [u8; ENCRYPTION_KEY_LEN];

impl User {
    fn get_key(&self) -> EncryptionKey {
        let mut salt = [0u8; ENCRYPTION_KEY_LEN];

        salt[0..self.username.as_bytes().len()].copy_from_slice(self.username.as_bytes());

        let iters = NonZeroU32::new(1000).unwrap();

        let mut key = EncryptionKey::default();

        ring::pbkdf2::derive(PBKDF2_ALG, iters, &salt, self.password.as_bytes(), &mut key);
        return key;
    }
}

#[derive(Debug)]
struct ObfuscatedFileName {
    obf_name: String,
}

fn get_hash(s: &str) -> String {
    format!(
        "{:?}",
        ring::digest::digest(&ring::digest::SHA512, s.as_bytes())
    )
    .chars()
    .skip(7)
    .collect()
}

impl From<(&User, &str)> for ObfuscatedFileName {
    fn from(inp: (&User, &str)) -> Self {
        let (user, file_name) = inp;
        let mut data = file_name.to_owned();
        data.push_str(&user.username);
        data.push_str(&user.password);
        ObfuscatedFileName {
            obf_name: get_hash(data.as_str()),
        }
    }
}

#[derive(Debug)]
struct EncryptedFile {
    name: ObfuscatedFileName,
    contents: String,
}

// Based on https://web3developer.io/authenticated-encryption-in-rust-using-ring/
struct CounterNonceSequence(u128);

impl From<u128> for CounterNonceSequence {
    fn from(value: u128) -> Self {
        CounterNonceSequence(value)
    }
}

impl NonceSequence for CounterNonceSequence {
    fn advance(&mut self) -> Result<Nonce, ring::error::Unspecified> {
        let mut nonce_bytes = vec![0; 12]; // Adjust the size based on your algorithm
        let bytes = self.0.to_be_bytes();
        nonce_bytes.copy_from_slice(&bytes[0..12]);
        self.0 = self.0.wrapping_mul(7);
        Nonce::try_assume_unique_for_key(&nonce_bytes)
    }
}

impl EncryptedFile {
    fn encrypt(
        user: &User,
        file_name: &str,
        plaintext: &str,
    ) -> Result<EncryptedFile, ring::error::Unspecified> {
        let key = user.get_key();
        let rng = ring::rand::SystemRandom::new();
        let mut nonce = [0u8; 16];
        rng.fill(&mut nonce)?;

        let unbound_key = UnboundKey::new(&ring::aead::AES_256_GCM, &key)?;

        let value: u128 = unsafe { std::mem::transmute(nonce) };
        let nonce_seq = CounterNonceSequence::from(value);

        let mut sealing_key = SealingKey::new(unbound_key, nonce_seq);

        let nonce_init_bytes = value.to_be_bytes();
        let mut in_out = plaintext.bytes().collect::<Vec<u8>>();

        sealing_key.seal_in_place_append_tag(Aad::from(user.username.as_bytes()), &mut in_out)?;
        in_out.extend_from_slice(&nonce_init_bytes);

        Ok(EncryptedFile {
            name: ObfuscatedFileName::from((user, file_name)),
            contents: base64::engine::general_purpose::STANDARD.encode(in_out),
        })
    }

    fn decrypt(&self, user: &User) -> Result<String, ring::error::Unspecified> {
        let mut decoded = base64::engine::general_purpose::STANDARD
            .decode(&self.contents)
            .unwrap();
        let mut nonce_init_bytes = [0u8; 16];
        let n = decoded.len();
        nonce_init_bytes.copy_from_slice(&decoded[n - 16..]);

        let value = u128::from_be_bytes(nonce_init_bytes);

        let cipher_text = &mut decoded[0..n - 16];
        let key = user.get_key();
        let unbound_key = UnboundKey::new(&ring::aead::AES_256_GCM, &key)?;
        let nonce_seq = CounterNonceSequence::from(value);
        let mut opening_key = OpeningKey::new(unbound_key, nonce_seq);

        let decrypted =
            opening_key.open_in_place(Aad::from(user.username.as_bytes()), cipher_text)?;

        Ok(std::str::from_utf8(decrypted).unwrap().to_owned())
    }
}
#[test]
fn encryption_decryption_works() {
    let data = "Hej";

    let user = User {
        username: "a".to_owned(),
        password: "a".to_owned(),
    };

    let encr = EncryptedFile::encrypt(&user, "hej", data).unwrap();
    let decr = encr.decrypt(&user).unwrap();

    assert_eq!(data, decr);
}

fn main() -> Result<()> {
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

    let store_files = |files: &HashMap<String, String>| -> Result<()> {
        let mut outp = String::new();
        for (file, hash) in files {
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

        Ok(())
    };

    let mut buffer = String::new();

    print!("Write your username: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut buffer)?;

    let username = buffer.trim().to_owned().clone();

    print!("Write your password: ");
    stdout().flush().unwrap();
    buffer.clear();
    stdin().read_line(&mut buffer)?;
    let password = buffer.trim().to_owned().clone();
    let user = User { username, password };

    println!(
        "logged in as '{}' with password '{}'",
        user.username, user.password
    );

    //This does not work on Windows.

    //L

    print!("Write the IP of the server: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut buffer)?;
    let addr = buffer.trim().to_owned().clone();

    //Uncommment the line below if you are on Windows
    // let addr = "127.0.0.1:31337";

    let stream = TcpStream::connect(addr)?;

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

                // TODO: get from server
                let obf_name = ObfuscatedFileName::from((&user, parts[1]));

                let data = match get_from_server(&stream, &obf_name.obf_name) {
                    Ok(response) => match response.data {
                        Some(data) => data,
                        None => {
                            println!("No file at file ID: {}", parts[1]);
                            continue;
                        }
                    },
                    Err(_) => {
                        println!("Couldn't get file at ID {}", parts[1]);
                        continue;
                    }
                };
                let from_server = EncryptedFile {
                    name: obf_name,
                    contents: data,
                };

                let decrypted = match from_server.decrypt(&user) {
                    Ok(file) => file,
                    Err(_) => {
                        println!("Decryption failed!");
                        continue;
                    }
                };
                file.write_all(decrypted.as_bytes())?;

                dbg!(&decrypted);

                // verify hash of file from server
                if let Some(r) = files.get_mut(get_hash(parts[1]).as_str()) {
                    if *r != get_hash(decrypted.as_str()) {
                        println!("FILE CONTENTS DO NOT MATCH");
                    }
                } else {
                    println!("WARNING, no local hash of file '{}'", parts[1]);
                    files.insert(get_hash(parts[1]), get_hash(decrypted.as_str()));
                    store_files(&files)?;
                }
            }
            "put" => {
                let mut file = std::fs::File::open(parts[1])?;
                let mut buffer = String::new();
                file.read_to_string(&mut buffer)?;
                files.insert(get_hash(parts[1]), get_hash(buffer.as_str()));

                let encrypted = EncryptedFile::encrypt(&user, parts[1], &buffer).unwrap();

                // TODO: send `encrypted` to server
                match send_to_server(&stream, &encrypted.name.obf_name, &encrypted.contents) {
                    Ok(_) => {
                        println!("File sent succesfully!");
                    }
                    Err(_) => {
                        println!("File transfer unsuccesfull")
                    }
                }

                // store hashes to disk
                store_files(&files)?;
            }
            _ => {
                dbg_usage();
            }
        }
    }
}
