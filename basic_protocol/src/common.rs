#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
use sodiumoxide;
use serde::{Serialize, Deserialize};

pub static SOCKET_PATH: &'static str = "/tmp/loopback-socket";

#[derive(Serialize, Deserialize, Debug)]
pub enum MyRequest {
    ReqCryptoBoxGenKeypair,
    ReqCryptoBoxGenNonce,
    ReqCryptoBoxSeal {
        keyid: usize,
        plaintext: Vec<u8>,
        public_key: sodiumoxide::crypto::box_::PublicKey,
    },
    ReqCryptoBoxOpen {
        keyid: usize,
        ciphertext: Vec<u8>,
        public_key: sodiumoxide::crypto::box_::PublicKey,
        nonce: sodiumoxide::crypto::box_::Nonce,
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MyResponse {
    ResCryptoBoxGenKeypair {
        keyid: usize,
        public_key: sodiumoxide::crypto::box_::PublicKey,
    },
    ResCryptoBoxGenNonce {
        nonce: sodiumoxide::crypto::box_::Nonce,
    },
    ResCryptoBoxSeal {
        ciphertext: Vec<u8>,
        nonce: sodiumoxide::crypto::box_::Nonce,
    },
    ResCryptoBoxOpen {
        plaintext: Vec<u8>,
    },
}

