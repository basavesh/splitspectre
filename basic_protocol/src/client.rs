#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
use std::os::unix::net::UnixStream;
use sodiumoxide;
use std::io::{self, BufRead, BufReader, Write};
use std::str::from_utf8;
use common::*;
use std::io::prelude::*;

pub mod common;

fn main() {

    let mut stream = UnixStream::connect(SOCKET_PATH).expect("Couldn't connect to the socket");

    let request = MyRequest::ReqCryptoBoxGenNonce;
    let request_string = serde_json::to_string(&request).unwrap();
    stream.write(request_string.as_bytes()).expect("Failed to write to server");
    let mut response = String::new();
    let _ = stream.read_to_string(&mut response).unwrap();
    let response: MyResponse = serde_json::from_str(&response).unwrap();

    if let MyResponse::ResCryptoBoxGenNonce{nonce} = response {
        println!("The Nonce is {:?}", nonce);
    } else {
        panic!("Things didn't go accordingly bro");
    }
}