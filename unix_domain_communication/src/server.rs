#![allow(unused_imports)]
use std::env;
use std::fs;
use std::thread;
use std::path::Path;
use std::str::from_utf8;
use common::SOCKET_PATH;
use std::collections::HashMap;
use std::io::{Read, Write, Error};
use std::sync::{Arc, Mutex, RwLock};
use std::os::unix::net::{UnixStream, UnixListener};

mod common;

fn handle_client(mut stream: UnixStream) -> Result<(), Error> {

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
                children.push(thread::spawn(|| handle_client(stream)));
            }
            Err(err) => {
                println!("Error:{}", err);
                break;
            }
        }
    }
}