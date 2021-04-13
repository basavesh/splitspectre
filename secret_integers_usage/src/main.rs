use std::io;
use simple::*;
pub mod simple;

fn main() {
    let secret_key: Vec<secret_integers::U8> = get_secret_key();
    let mut buffer: String = String::new();
    println!("Please input a 8 byte message");
    let _ = io::stdin().read_line(&mut buffer);
    let mut message = [0u8; 8];
    for i in 0..8 {
        if i < buffer.len() {
            message[i] = buffer.as_bytes()[i];
        }
    }
    let cipher_text: Vec<u8> = encrypt(&message, &secret_key);
    let text: Vec<u8> = decrypt(&cipher_text, &secret_key);
    assert!(message == &text[..]);
    println!("message: {:?}", message);
    println!("ciphertext: {:?}", cipher_text);
}
