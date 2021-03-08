use std::io;
use agent_client::*;
pub mod agent_client;
pub mod agent_common;

fn main() {
    let secret_key = agent_get_secret_key().unwrap();
    println!("Secret Key id is {}", secret_key);
    let mut buffer = String::new();
    println!("Please input a 8 byte message");
    let _ = io::stdin().read_line(&mut buffer);
    let mut message = [0u8; 8];
    for i in 0..8 {
        if i < buffer.len() {
            message[i] = buffer.as_bytes()[i];
        }
    }
    let cipher_text = agent_encrypt(&message, &secret_key).unwrap();
    let text = agent_decrypt(&cipher_text, &secret_key).unwrap();
    assert!(message == &text[..]);
    println!("message: {:?}", message);
    println!("ciphertext: {:?}", cipher_text);
}