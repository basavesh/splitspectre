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

pub fn get_secret_key() -> Vec<U8> {
    let random_bytes = rand::thread_rng().gen::<[u8; 8]>();
    return classify_u8s(&random_bytes);
}

pub fn encrypt(msg: &[u8], sk: &[U8]) -> Vec<u8> {
    let mut new_block = [U8::zero(); 8];
    let classified_msg = classify_u8s(msg);
    for i in 0..8 {
        new_block[i] = classified_msg[i] ^ sk[i];
    }
    return declassify_u8s(&new_block);
}

pub fn decrypt(cipher: &[u8], sk: &[U8]) -> Vec<u8> {
    let mut new_block = [U8::zero(); 8];
    let classified_cipher = classify_u8s(cipher);
    for i in 0..8 {
        new_block[i] = classified_cipher[i] ^ sk[i];
    }
    return declassify_u8s(&new_block);
}
