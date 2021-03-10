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
use simple::*;

pub mod agent_common;
pub mod simple;

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
                write_guard.insert(*num, get_secret_key());

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
                    let new_block = encrypt(&plaintext, &sk);

                    let response = MyResponse::ResEncrypt{ ciphertext: new_block};
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
                    let new_block = decrypt(&ciphertext, &sk);

                    let response = MyResponse::ResDecrypt{ plaintext: new_block};
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
                children.push(thread::spawn(move ||
                    handle_client(stream, child_arc_keys_map, child_arc_counter)));
            }

            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }

    for child in children {
        let _ = child.join();
    }
}