use std::fs;
use std::thread;
use std::path::Path;
use agent_common::*;
use std::collections::HashMap;
use std::io::{Read, Write, Error};
use std::sync::{Arc, Mutex, RwLock};
use std::os::unix::net::{UnixStream, UnixListener};
use std::net::Shutdown;
use secret_integers::*;
use rand::Rng;

pub mod agent_common;

/// classify vector of u8s into U8s
fn classify_u8s(v: &[u8]) -> Vec<U8> {
    v.iter().map(|x| U8::classify(*x)).collect()
}

/// declassify vector of U8s into u8s
fn declassify_u8s(v: &[U8]) -> Vec<u8> {
    v.iter().map(|x| U8::declassify(*x)).collect()
}

fn handle_client(mut stream: UnixStream, child_arc_keys_map: Arc<RwLock<HashMap<u64,
    Vec<U8>>>>, child_arc_counter: Arc<Mutex<u64>>) -> Result<(), Error> {

    let mut buf = [0; 4096];        // Need to check how to do it with Vectors

    let bytes_read = stream.read(&mut buf)?;
    if bytes_read == 0 {return Ok(())}
    if let Ok(request) = std::str::from_utf8(&buf[..bytes_read]) {

        let request: MyRequest = serde_json::from_str(&request).unwrap();

        // GenSecretKey
        if let MyRequest::ReqGetSecretKey = request {

            if let Ok(mut write_guard) = child_arc_keys_map.write() {
                let mut num = child_arc_counter.lock().unwrap();
                *num += 1;
                write_guard.insert(*num, classify_u8s(&rand::thread_rng().gen::<[u8; 8]>().to_vec()));

                let response = MyResponse::ResGetSecretKey{ keyid: *num};
                let response_string = serde_json::to_string(&response).unwrap();
                stream.write_all(response_string.as_bytes()).expect("IO Error");
                stream.shutdown(Shutdown::Both).expect("shutdown function failed");
            }
        }

        // Encrypt
        if let MyRequest::ReqEncrypt{ref plaintext, keyid} = request {

            if let Ok(read_guard) = child_arc_keys_map.read() {

                if read_guard.contains_key(&keyid) {
                    let sk = &read_guard[&keyid];
                    let mut new_block = [U8::zero(); 8];
                    let classified_msg = classify_u8s(&plaintext);
                    for i in 0..8 {
                        new_block[i] = classified_msg[i] ^ sk[i];
                    }

                    let response = MyResponse::ResEncrypt{ ciphertext: declassify_u8s(&new_block)};
                    let response_string = serde_json::to_string(&response).unwrap();
                    stream.write_all(response_string.as_bytes()).expect("IO Error");
                    stream.shutdown(Shutdown::Both).expect("shutdown function failed");
                }
            }
        }


        // Decrypt
        if let MyRequest::ReqDecrypt{ ciphertext, keyid} = request {

            if let Ok(read_guard) = child_arc_keys_map.read() {

                if read_guard.contains_key(&keyid) {
                    let sk = &read_guard[&keyid];
                    let mut new_block = [U8::zero(); 8];
                    let classified_msg = classify_u8s(&ciphertext);
                    for i in 0..8 {
                        new_block[i] = classified_msg[i] ^ sk[i];
                    }

                    let response = MyResponse::ResDecrypt{ plaintext: declassify_u8s(&new_block)};
                    let response_string = serde_json::to_string(&response).unwrap();
                    stream.write_all(response_string.as_bytes()).expect("IO Error");
                    stream.shutdown(Shutdown::Both).expect("shutdown function failed");
                }
            }
        }

    }

    return Ok(());
}


fn main() {
    println!("Hello World");

    let socket = Path::new(SOCKET_PATH);
    let mut children = vec![];
    let keys_map: HashMap<u64, Vec<U8>> = HashMap::new();
    let arc_keys_map = Arc::new(RwLock::new(keys_map));

    let counter: u64 = 0;
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