use std::io;
use rand::Rng;
use secret_integers::*;

/// classify vector of u8s into U8s
fn classify_u8s(v: &[u8]) -> Vec<U8> {
    v.iter().map(|x| U8::classify(*x)).collect()
}

/// declassify vector of U8s into u8s
fn declassify_u8s(v: &[U8]) -> Vec<u8> {
    v.iter().map(|x| U8::declassify(*x)).collect()
}

fn encrypt(msg: &[u8], sk: &[U8]) -> Vec<u8> {
    let mut new_block = [U8::zero(); 8];
    let classified_msg = classify_u8s(msg);
    for i in 0..8 {
        new_block[i] = classified_msg[i] ^ sk[i];
    }

    return declassify_u8s(&new_block);
}

fn decrypt(cipher: &[u8], sk: &[U8]) -> Vec<u8> {
    let mut new_block = [U8::zero(); 8];
    let classified_cipher = classify_u8s(cipher);
    for i in 0..8 {
        new_block[i] = classified_cipher[i] ^ sk[i];
    }
    return declassify_u8s(&new_block);
}

fn main() {
    let random_bytes = rand::thread_rng().gen::<[u8; 8]>();
    let secret_key = classify_u8s(&random_bytes);
    // let message = rand::thread_rng().gen::<[u8; 8]>();
    let mut buffer = String::new();
    let _ = io::stdin().read_line(&mut buffer);
    let mut message = [0u8; 8];
    for i in 0..8 {
        if i < buffer.len() {
            message[i] = buffer.as_bytes()[i];
        }
    }
    let cipher_text = encrypt(&message, &secret_key);
    let text = decrypt(&cipher_text, &secret_key);
    assert!(message == &text[..]);
    println!("message: {:?}", message);
    println!("ciphertext: {:?}", cipher_text);
}