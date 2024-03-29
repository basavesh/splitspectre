use sodiumoxide::crypto::*;

fn use_box_basic () {
    let (ourpk, oursk) = box_::gen_keypair();
    // normally theirpk is sent by the other party
    let (theirpk, theirsk) = box_::gen_keypair();
    let nonce = box_::gen_nonce();
    let plaintext = b"some data";
    let ciphertext = box_::seal(plaintext, &nonce, &theirpk, &oursk);
    let their_plaintext = box_::open(&ciphertext, &nonce, &ourpk, &theirsk).unwrap();
    assert!(plaintext == &their_plaintext[..]);

}


fn use_box_seal_loop() {
    // our three encrypting key pairs
    let (ourpk1, oursk1) = box_::gen_keypair();
    let (ourpk2, oursk2) = box_::gen_keypair();
    let (ourpk3, oursk3) = box_::gen_keypair();

    let pk_keys = vec![ourpk1, ourpk2, ourpk3];
    let sk_keys = vec![oursk1, oursk2, oursk3];
    let (theirpk, theirsk) = box_::gen_keypair();

    let plaintext: Vec<u8> = b"Encrypt this data".to_vec();

    let mut ciphertext = plaintext.clone();
    let nonces = vec![box_::gen_nonce(), box_::gen_nonce(), box_::gen_nonce()];
    for i in 0..3 {
        ciphertext = box_::seal(&ciphertext, &nonces[i], &theirpk, &sk_keys[i]);
    }

    let mut their_plaintext = ciphertext.clone();
    for i in 0..3 {
        their_plaintext = box_::open(&their_plaintext, &nonces[2 - i], &pk_keys[2 - i], &theirsk).unwrap();
    }

    assert!(plaintext == &their_plaintext[..]);
}


fn main () {

    // basic usage
    use_box_basic();

    // Create some 3 Encryption Key pairs and seal and open
    use_box_seal_loop();
}