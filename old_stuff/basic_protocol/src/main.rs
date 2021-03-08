#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
use serde::{Serialize, Deserialize};
use sodiumoxide;
use std::any::type_name;
use serde_json::Result;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
enum MyRequest {
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
enum MyResponse {
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

#[derive(Debug)]
struct KeyPair {
    public_key: sodiumoxide::crypto::box_::PublicKey,
    secret_key: sodiumoxide::crypto::box_::SecretKey, 
}

fn handle_request(request: String, keys_map: &mut HashMap<usize, KeyPair>, counter: &mut usize) -> Option<String> {
    let request = serde_json::from_str(&request).unwrap();

    if let MyRequest::ReqCryptoBoxGenKeypair = request {
        let (public_key, secret_key) = sodiumoxide::crypto::box_::gen_keypair();
        *counter += 1;
        keys_map.insert(*counter, KeyPair{ public_key, secret_key});
        let response = MyResponse::ResCryptoBoxGenKeypair{ keyid: *counter, public_key};
        return Some(serde_json::to_string(&response).unwrap());
    }

    if let MyRequest::ReqCryptoBoxGenNonce = request {
        let new_nonce = sodiumoxide::crypto::box_::gen_nonce();
        let response = MyResponse::ResCryptoBoxGenNonce{ nonce: new_nonce};
        return Some(serde_json::to_string(&response).unwrap());
    }
    
    if let MyRequest::ReqCryptoBoxSeal{ keyid, plaintext, public_key} = request {
        if keys_map.contains_key(&keyid) {
            let nonce = sodiumoxide::crypto::box_::gen_nonce();
            let ciphertext = sodiumoxide::crypto::box_::seal(&plaintext, &nonce, &public_key, &keys_map[&keyid].secret_key);
            let response = MyResponse::ResCryptoBoxSeal{ ciphertext, nonce};
            return Some(serde_json::to_string(&response).unwrap());
        } 
        return None;
    }

    if let MyRequest::ReqCryptoBoxOpen{keyid, ciphertext, public_key, nonce } = request {
        if keys_map.contains_key(&keyid) {
            if let Ok(plaintext) = sodiumoxide::crypto::box_::open(&ciphertext, &nonce, &public_key, &keys_map[&keyid].secret_key) {
                let response = MyResponse::ResCryptoBoxOpen{plaintext};
                return Some(serde_json::to_string(&response).unwrap());
            }
            return None;
        } 
        return None;
    }
    return None;
}


fn main() {
    let mut keys_map: HashMap<usize, KeyPair> = HashMap::new();         // will need to wrap around Arc Rwlock later
    let mut counter: usize = 0;                                         // will need to wrap around Arc mutex later

    {
        // Generate a nonce for fun
        let request = MyRequest::ReqCryptoBoxGenNonce;
        let request_string = serde_json::to_string(&request).unwrap();
        let response_string = handle_request(request_string,&mut keys_map, &mut counter).unwrap(); 
        let response: MyResponse = serde_json::from_str(&response_string).unwrap();
        if let MyResponse::ResCryptoBoxGenNonce{nonce} = response {
            // println!("The Nonce is {:?}", nonce);
        } else {
            panic!("Things didn't go accordingly bro");
        }
    }

    // Generate a KeyPair1
    let request = MyRequest::ReqCryptoBoxGenKeypair;
    let request_string = serde_json::to_string(&request).unwrap();
    let response_string = handle_request(request_string,&mut keys_map, &mut counter).unwrap(); 
    let response: MyResponse = serde_json::from_str(&response_string).unwrap();
    let my_key_id1: usize;
    let public_key1: sodiumoxide::crypto::box_::PublicKey;
    if let MyResponse::ResCryptoBoxGenKeypair{keyid, public_key} = response {
        // println!("Hurray?");
        my_key_id1 = keyid;
        public_key1 = public_key;
    } else {
        panic!("Things didn't go accordingly bro");
    }
    // println!("My KeyId is {}", my_key_id1);
    // println!("My Public_key is {:?}", public_key1);

    // trying a little hack here
    let (theirpk, theirsk) = sodiumoxide::crypto::box_::gen_keypair();
    let plaintext = b"Encrypt this data";
    let request = MyRequest::ReqCryptoBoxSeal{keyid: my_key_id1, plaintext: plaintext.to_vec(), public_key: theirpk};
    let request_string = serde_json::to_string(&request).unwrap();
    let response_string = handle_request(request_string,&mut keys_map, &mut counter).unwrap(); 
    let response: MyResponse = serde_json::from_str(&response_string).unwrap();
    if let MyResponse::ResCryptoBoxSeal{ciphertext, nonce} = response {
        if let Ok(their_plaintext) = sodiumoxide::crypto::box_::open(&ciphertext, &nonce, &public_key1, &theirsk) {
            // println!("Testing the decryption part");
            assert!(plaintext == &their_plaintext[..]);
            println!("Decryption1 successful!!!")
        } else {
            panic!("Decryption1 failed");
        }
        
    } else {
        panic!("Something is wrong!!!");
    }

    // Generate a KeyPair2
    let request = MyRequest::ReqCryptoBoxGenKeypair;
    let request_string = serde_json::to_string(&request).unwrap();
    let response_string = handle_request(request_string,&mut keys_map, &mut counter).unwrap(); 
    let response: MyResponse = serde_json::from_str(&response_string).unwrap();
    let my_key_id2: usize;
    let public_key2: sodiumoxide::crypto::box_::PublicKey;
    if let MyResponse::ResCryptoBoxGenKeypair{keyid, public_key} = response {
        // println!("Hurray?");
        my_key_id2 = keyid;
        public_key2 = public_key;
    } else {
        panic!("Things didn't go accordingly bro");
    }
    
    // Let's encrypt and decrypt using the wrapper functions. 
    // Client1 sending data to Client2 and Client2 decrypts the message
    let plaintext = b"Encrypt this data";
    let request = MyRequest::ReqCryptoBoxSeal{keyid: my_key_id1, plaintext: plaintext.to_vec(), public_key: public_key2};
    let request_string = serde_json::to_string(&request).unwrap();
    let response_string = handle_request(request_string,&mut keys_map, &mut counter).unwrap(); 
    let response: MyResponse = serde_json::from_str(&response_string).unwrap();
    if let MyResponse::ResCryptoBoxSeal{ciphertext, nonce} = response {
        // client 2 will decrypt now
        let request = MyRequest::ReqCryptoBoxOpen{keyid: my_key_id2, ciphertext, nonce, public_key: public_key1};
        let request_string = serde_json::to_string(&request).unwrap();
        let response_string = handle_request(request_string,&mut keys_map, &mut counter).unwrap(); 
        let response: MyResponse = serde_json::from_str(&response_string).unwrap();
        if let MyResponse::ResCryptoBoxOpen{plaintext: their_plaintext} = response {
            assert!(plaintext == &their_plaintext[..]);
            println!("Decryption2 successful!!!");
        }
    } else {
        panic!("Something is wrong!!!");
    }
}
