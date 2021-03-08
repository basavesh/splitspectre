use std::os::unix::net::UnixStream;
use std::io::prelude::*;
use std::net::Shutdown;
use crate::agent_common::*;
use std::io::Write;

pub fn agent_get_secret_key() -> Option<u64> {
    let mut stream = UnixStream::connect(SOCKET_PATH)
                        .expect("Couldn't connect to the socket");
    let request = MyRequest::ReqGetSecretKey;
    let request_string = serde_json::to_string(&request).unwrap();
    stream.write(request_string.as_bytes()).expect("Failed to write to server");
    let mut response = String::new();
    let _ = stream.read_to_string(&mut response).unwrap();
    let response: MyResponse = serde_json::from_str(&response).unwrap();
    stream.shutdown(Shutdown::Both).expect("shutdown function failed");

    if let MyResponse::ResGetSecretKey{keyid} = response {
        return Some(keyid);
    } else {
        return None;
    }
}

pub fn agent_encrypt(msg: &[u8], sk: &u64) -> Option<Vec<u8>> {
    let mut stream = UnixStream::connect(SOCKET_PATH)
                        .expect("Couldn't connect to the socket");
    let request = MyRequest::ReqEncrypt{
        plaintext: msg.to_vec(),
        keyid: *sk,
    };
    let request_string = serde_json::to_string(&request).unwrap();
    stream.write(request_string.as_bytes()).expect("Failed to write to server");
    let mut response = String::new();
    let _ = stream.read_to_string(&mut response).unwrap();
    let response: MyResponse = serde_json::from_str(&response).unwrap();

    if let MyResponse::ResEncrypt{ciphertext} = response {
        return Some(ciphertext);
    }
    return None;
}

pub fn agent_decrypt(cipher: &[u8], sk: &u64) -> Option<Vec<u8>> {

    let mut stream = UnixStream::connect(SOCKET_PATH)
                        .expect("Couldn't connect to the socket");
    let request = MyRequest::ReqDecrypt{
        ciphertext: cipher.to_vec(),
        keyid: *sk,
    };
    let request_string = serde_json::to_string(&request).unwrap();
    stream.write(request_string.as_bytes()).expect("Failed to write to server");
    let mut response = String::new();
    let _ = stream.read_to_string(&mut response).unwrap();
    let response: MyResponse = serde_json::from_str(&response).unwrap();

    if let MyResponse::ResDecrypt{plaintext} = response {
        return Some(plaintext);
    }
    return None;
}
