#![allow(unused_imports)]
#![allow(dead_code)]
use std::env;
use std::fs;
use std::thread;
use std::path::Path;
use std::str::from_utf8;
use common::SOCKET_PATH;
use sodiumoxide::crypto::box_;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::io::{Read, Write, Error};
use std::sync::{Arc, Mutex, RwLock};
use std::os::unix::net::{UnixStream, UnixListener};

mod common;

#[derive(Hash, Eq, PartialEq, Debug, Serialize, Deserialize)]
struct SecretData {
    secret_key: String,
    public_key: String,
}

impl SecretData {
    fn new(secret: &str, public: &str) -> SecretData {
        SecretData {
            secret_key: secret.to_string(),
            public_key: public.to_string()
        }
    }
}

fn handle_client(mut stream: UnixStream,
                 child_arc_keys_map: Arc<RwLock<HashMap<usize, SecretData>>>, 
                 id: &usize) -> Result<(), Error> {

    if let Ok(mut write_guard) = child_arc_keys_map.write() {
        write_guard.insert(*id, SecretData::new( "child secret stuff", "child public stuff"));
        println!("Write_guard val is {:?}", *write_guard);
    };

    {
        // Example code related to sodium oxide thing
        let (ourpk, oursk) = box_::gen_keypair();
        // normally theirpk is sent by the other party
        let (theirpk, theirsk) = box_::gen_keypair();
        let nonce = box_::gen_nonce();
        let plaintext = b"some data";
        let ciphertext = box_::seal(plaintext, &nonce, &theirpk, &oursk);
        let their_plaintext = box_::open(&ciphertext, &nonce, &ourpk, &theirsk).unwrap();
        assert!(plaintext == &their_plaintext[..]);
    }

    let mut buf = [0; 512];
    loop {
        let bytes_read = stream.read(&mut buf)?;
        if bytes_read == 0 { return Ok(()) }
        stream.write(&buf[..bytes_read])?;
    }
}


fn main() {

    let socket = Path::new(SOCKET_PATH);
    let mut children = vec![];

    // TODO change the hashmap type later once the protocol is decided.
    let keys_map: HashMap<usize, SecretData> = HashMap::new();
    let arc_keys_map = Arc::new(RwLock::new(keys_map));

    // Delete old socket if necessary
    if socket.exists() {
        fs::remove_file(&socket).unwrap();
    }

    let mut id_gen: usize = 0;

    let listener = UnixListener::bind(&socket).unwrap();

    for stream in listener.incoming() {
        // Returns a stream of incoming connections.
        // Iterating over this stream is equivalent to calling accept in a loop
        match stream {
            Ok(stream) => {
                println!("got connection request");
                let child_arc_keys_map = arc_keys_map.clone();
                // let child_id_gen = id_gen.clone();
                id_gen += 1; // simple code will change this later
                children.push(thread::spawn(move || handle_client(stream, child_arc_keys_map, &id_gen)));
            }
            Err(err) => {
                println!("Error:{}", err);
                break;
            }
        }
    }

    for child in children {
        let _ = child.join();
    }
}
