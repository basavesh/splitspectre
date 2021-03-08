#![allow(unused_imports)]
use std::os::unix::net::UnixStream;
use sodiumoxide::crypto::box_;
use std::io::{self, BufRead, BufReader, Write};
use std::str::from_utf8;
use common::SOCKET_PATH;

pub mod common;


fn main() {

    let mut stream = UnixStream::connect(SOCKET_PATH).expect("Couldn't connect to the socket");

    loop {
        let mut input = String::new();
        let mut buffer: Vec<u8> = Vec::new();
        io::stdin().read_line(&mut input).expect("Failed to read from stdin");
        stream.write(input.as_bytes()).expect("Failed to write to server");

        let mut reader = BufReader::new(&stream);

        reader.read_until(b'\n', &mut buffer).expect("Could not read into buffer");
        print!("{}", from_utf8(&buffer).expect("Could not write buffer as string"));
        //println!("{}", from_utf8(&buffer).unwrap());
    }
    
}
