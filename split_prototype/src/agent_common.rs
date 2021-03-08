use serde::{Serialize, Deserialize};

pub static SOCKET_PATH: &'static str = "/tmp/loopback-socket";

#[derive(Serialize, Deserialize, Debug)]
pub enum MyRequest {
    ReqGetSecretKey,
    ReqEncrypt {
        plaintext: Vec<u8>,
        keyid: u64,
    },
    ReqDecrypt {
        ciphertext: Vec<u8>,
        keyid: u64,
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MyResponse {
    ResGetSecretKey {
        keyid: u64,
    },
    ResEncrypt {
        ciphertext: Vec<u8>,
    },
    ResDecrypt {
        plaintext: Vec<u8>,
    }
}