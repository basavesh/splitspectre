#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
use std::env;
use std::fs;
use std::thread;
use std::path::Path;
use std::str::from_utf8;
use basic_common::SOCKET_PATH;
use basic_common::*;
use sodiumoxide;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::io::{Read, Write, Error};
use std::sync::{Arc, Mutex, RwLock};
use std::io::{self, BufRead, BufReader};
use std::os::unix::net::{UnixStream, UnixListener};
use std::net::Shutdown;


pub mod basic_common;

#[derive(Debug)]
struct KeyPair {
    public_key: sodiumoxide::crypto::box_::PublicKey,
    secret_key: sodiumoxide::crypto::box_::SecretKey,
}

fn trim_newline(s: &mut String) {
    while s.ends_with('\n') || s.ends_with('\r') {
        s.pop();
    }
}

fn handle_client(mut stream: UnixStream, child_arc_keys_map: Arc<RwLock<HashMap<usize, KeyPair>>>, child_arc_counter: Arc<Mutex<usize>>) -> Result<(), Error> {

    let mut buf = [0; 4096];        // Need to check how to do it with Vectors
    loop {
        let bytes_read = stream.read(&mut buf)?;
        if bytes_read == 0 {return Ok(())}
        if let Ok(request) = std::str::from_utf8(&buf[..bytes_read]) {

            let request: MyRequest = serde_json::from_str(&request).unwrap();

            if let MyRequest::ReqCryptoBoxGenNonce = request {
                let new_nonce = sodiumoxide::crypto::box_::gen_nonce();
                let response = MyResponse::ResCryptoBoxGenNonce{ nonce: new_nonce};
                let response_string = serde_json::to_string(&response).unwrap();
                stream.write_all(response_string.as_bytes()).expect("IO Error");
                stream.shutdown(Shutdown::Both).expect("shutdown function failed");
            }

            // TODO for other cases



        }
    }
}

// fn handle_request(request: String, keys_map: &mut HashMap<usize, KeyPair>, counter: &mut usize) -> Option<String> {
//     let request = serde_json::from_str(&request).unwrap();

//     if let MyRequest::ReqCryptoBoxGenKeypair = request {
//         let (public_key, secret_key) = sodiumoxide::crypto::box_::gen_keypair();
//         *counter += 1;
//         keys_map.insert(*counter, KeyPair{ public_key, secret_key});
//         let response = MyResponse::ResCryptoBoxGenKeypair{ keyid: *counter, public_key};
//         return Some(serde_json::to_string(&response).unwrap());
//     }

//     if let MyRequest::ReqCryptoBoxGenNonce = request {
//         let new_nonce = sodiumoxide::crypto::box_::gen_nonce();
//         let response = MyResponse::ResCryptoBoxGenNonce{ nonce: new_nonce};
//         return Some(serde_json::to_string(&response).unwrap());
//     }

//     if let MyRequest::ReqCryptoBoxSeal{ keyid, plaintext, public_key} = request {
//         if keys_map.contains_key(&keyid) {
//             let nonce = sodiumoxide::crypto::box_::gen_nonce();
//             let ciphertext = sodiumoxide::crypto::box_::seal(&plaintext, &nonce, &public_key, &keys_map[&keyid].secret_key);
//             let response = MyResponse::ResCryptoBoxSeal{ ciphertext, nonce};
//             return Some(serde_json::to_string(&response).unwrap());
//         }
//         return None;
//     }

//     if let MyRequest::ReqCryptoBoxOpen{keyid, ciphertext, public_key, nonce } = request {
//         if keys_map.contains_key(&keyid) {
//             if let Ok(plaintext) = sodiumoxide::crypto::box_::open(&ciphertext, &nonce, &public_key, &keys_map[&keyid].secret_key) {
//                 let response = MyResponse::ResCryptoBoxOpen{plaintext};
//                 return Some(serde_json::to_string(&response).unwrap());
//             }
//             return None;
//         }
//         return None;
//     }
//     return None;
// }


fn main() {

    let socket = Path::new(SOCKET_PATH);
    let mut children = vec![];
    let keys_map: HashMap<usize, KeyPair> = HashMap::new();
    let arc_keys_map = Arc::new(RwLock::new(keys_map));

    let counter: usize = 0;
    let arc_counter = Arc::new(Mutex::new(counter));

    // Delete old socket if necessary
    if socket.exists() {
        fs::remove_file(&socket).unwrap();
    }

    let listener = UnixListener::bind(&socket).unwrap();

    for stream in listener.incoming() {
        // Returns a stream of incoming connections.
        // Iterating over this stream is equivalent to calling accept in a loop
        match stream {
            Ok(stream) => {
                println!("got connection request");
                let child_arc_keys_map = arc_keys_map.clone();
                let child_arc_counter = arc_counter.clone();
                children.push(thread::spawn(move || handle_client(stream, child_arc_keys_map, child_arc_counter)));
            }

            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }


}
