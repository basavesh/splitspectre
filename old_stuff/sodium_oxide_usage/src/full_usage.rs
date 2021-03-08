use sodiumoxide::crypto::*;


fn use_box_seal() {
    let (ourpk, oursk) = box_::gen_keypair();
    // normalll their pk is sent by the other party
    let (theirpk, theirsk) = box_::gen_keypair();

    let nonce = box_::gen_nonce();
    let plaintext = b"Encrypt this data";
    let ciphertext = box_::seal(plaintext, &nonce, &theirpk, &oursk);
    let their_plaintext = box_::open(&ciphertext, &nonce, &ourpk, &theirsk).unwrap();
    assert!(plaintext == &their_plaintext[..]);
}


fn use_crypto_sign() {
    let (pk, sk) = sign::gen_keypair();
    let data_to_sign = b"Sign this data";
    let signed_data = sign::sign(data_to_sign, &sk);
    let verified_data = sign::verify(&signed_data, &pk).unwrap();
    assert!(data_to_sign == &verified_data[..]);
}


fn use_crypto_hash() {
    let data_to_hash = b"some data";
    let digest1 = hash::hash(data_to_hash);
    // println!("{:#?}", digest1);

    let mut hash_state = hash::State::new();
    hash_state.update(b"some ");
    hash_state.update(b"data");
    let digest2 = hash_state.finalize();
    // println!("{:#?}", digest2);
    assert!(digest1 == digest2);
}


fn use_crypto_aed() {
    let k = aead::gen_key();
    let n = aead::gen_nonce();
    let m = b"Some plaintext";
    let ad = b"Some additional data";

    let c = aead::seal(m, Some(ad), &n, &k);
    let m2 = aead::open(&c, Some(ad), &n, &k).unwrap();
    assert_eq!(&m[..], &m2[..]);
}


fn use_crypto_auth_simple() {
    let key = auth::gen_key();
    let data_to_authenticate = b"some data";
    let tag = auth::authenticate(data_to_authenticate, &key);
    assert!(auth::verify(&tag, data_to_authenticate, &key));
}


fn use_crypto_auth_streaming() {
    use sodiumoxide::randombytes;
    let key = randombytes::randombytes(123);

    let data_part_1 = b"some data";
    let data_part_2 = b"some other data";
    let mut state = auth::State::init(&key);
    state.update(data_part_1);
    state.update(data_part_2);
    let tag1 = state.finalize();

    let data_2_part_1 = b"some datasome ";
    let data_2_part_2 = b"other data";
    let mut state = auth::State::init(&key);
    state.update(data_2_part_1);
    state.update(data_2_part_2);
    let tag2 = state.finalize();
    assert_eq!(tag1, tag2);
}


fn use_crypto_kdf() {
    const CONTEXT: [u8; 8] = *b"Examples";
    let key = kdf::gen_key();
    // println!("{:#?}", key);

    let mut key1 = secretbox::Key([0; secretbox::KEYBYTES]);
    kdf::derive_from_key(&mut key1.0[..], 1, CONTEXT, &key).unwrap();
    
    let mut key2 = secretbox::Key([0; secretbox::KEYBYTES]);
    kdf::derive_from_key(&mut key2.0[..], 2, CONTEXT, &key).unwrap();
    
    let mut key3 = secretbox::Key([0; secretbox::KEYBYTES]);
    kdf::derive_from_key(&mut key3.0[..], 3, CONTEXT, &key).unwrap();
    // println!("{:#?}", key3);
}


fn use_crypto_kx() {
    // client-side
    let (client_pk, client_sk) = kx::gen_keypair();

    // server-side
    let (server_pk, server_sk) = kx::gen_keypair();

    // client and server exchanges client_pk and server_pk

    // client deduces the two session keys rx1 and tx1
    let (rx1, tx1) = match kx::client_session_keys(&client_pk, &client_sk, &server_pk) {
        Ok((rx, tx)) => (rx, tx),
        Err(()) => panic!("bad server signature"),
    };

    // server performs the same operation
    let (rx2, tx2) = match kx::server_session_keys(&server_pk, &server_sk, &client_pk) {
        Ok((rx, tx)) => (rx, tx),
        Err(()) => panic!("bad client signature"),
    };

    assert!(rx1==tx2);
    assert!(rx2==tx1);
}


fn use_crypto_onetimeauth() {
    let key = onetimeauth::gen_key();
    let data_to_authenticate = b"some data";
    let tag = onetimeauth::authenticate(data_to_authenticate, &key);
    assert!(onetimeauth::verify(&tag, data_to_authenticate, &key));
}


fn use_crypto_pwhash_verify() {
    let passwd = b"Correct Horse Battery Staple";
    // in reality we want to load the password hash from somewhere
    // and we might want to create a `HashedPassword` from it using
    // `HashedPassword::from_slice(pwhash_bytes).unwrap()`
    let pwh = pwhash::pwhash(passwd,
                            pwhash::OPSLIMIT_INTERACTIVE,
                            pwhash::MEMLIMIT_INTERACTIVE).unwrap();
    assert!(pwhash::pwhash_verify(&pwh, passwd));
}


fn use_crypto_scalarmult() {
    let bobsk = scalarmult::Scalar([
        0x5d, 0xab, 0x08, 0x7e, 0x62, 0x4a, 0x8a, 0x4b, 0x79, 0xe1, 0x7f, 0x8b, 0x83, 0x80,
        0x0e, 0xe6, 0x6f, 0x3b, 0xb1, 0x29, 0x26, 0x18, 0xb6, 0xfd, 0x1c, 0x2f, 0x8b, 0x27,
        0xff, 0x88, 0xe0, 0xeb,
    ]);
    let alicepk = scalarmult::GroupElement([
        0x85, 0x20, 0xf0, 0x09, 0x89, 0x30, 0xa7, 0x54, 0x74, 0x8b, 0x7d, 0xdc, 0xb4, 0x3e,
        0xf7, 0x5a, 0x0d, 0xbf, 0x3a, 0x0d, 0x26, 0x38, 0x1a, 0xf4, 0xeb, 0xa4, 0xa9, 0x8e,
        0xaa, 0x9b, 0x4e, 0x6a,
    ]);
    let k_expected = [
        0x4a, 0x5d, 0x9d, 0x5b, 0xa4, 0xce, 0x2d, 0xe1, 0x72, 0x8e, 0x3b, 0xf4, 0x80, 0x35,
        0x0f, 0x25, 0xe0, 0x7e, 0x21, 0xc9, 0x47, 0xd1, 0x9e, 0x33, 0x76, 0xf0, 0x9b, 0x3c,
        0x1e, 0x16, 0x17, 0x42,
    ];

    let scalarmult::GroupElement(k) = scalarmult::scalarmult(&bobsk, &alicepk).unwrap();
    assert!(k == k_expected);
}


fn use_crypto_secretbox() {
    let key = secretbox::gen_key();
    let nonce = secretbox::gen_nonce();
    let plaintext = b"some data";
    let ciphertext = secretbox::seal(plaintext, &nonce, &key);
    let their_plaintext = secretbox::open(&ciphertext, &nonce, &key).unwrap();
    assert!(plaintext == &their_plaintext[..]);
}


fn use_crytp_stream() {
    {
        // KeyStream generation
        let key = stream::gen_key();
        let nonce = stream::gen_nonce();
        let _keystream = stream::stream(128, &nonce, &key); // generate 128 bytes of keystream
    }

    {
        // Encryption
        let key = stream::gen_key();
        let nonce = stream::gen_nonce();
        let plaintext = b"some data";
        let ciphertext = stream::stream_xor(plaintext, &nonce, &key);
        let their_plaintext = stream::stream_xor(&ciphertext, &nonce, &key);
        assert_eq!(plaintext, &their_plaintext[..]);
    }

    {
        // In place encryption
        let key = stream::gen_key();
        let nonce = stream::gen_nonce();
        let plaintext = &mut [0, 1, 2, 3];
        // encrypt the plaintext
        stream::stream_xor_inplace(plaintext, &nonce, &key);
        // decrypt the plaintext
        stream::stream_xor_inplace(plaintext, &nonce, &key);
        assert_eq!(plaintext, &mut [0, 1, 2, 3]);
    }
}


fn use_crypto_secretstream() {
    use secretstream::{gen_key, Stream, Tag};

    let msg1 = "some message 1";
    let msg2 = "other message";
    let msg3 = "final message";
    
    // initialize encrypt secret stream
    let key = gen_key();
    let (mut enc_stream, header) = Stream::init_push(&key).unwrap();
    
    // encrypt first message, tagging it as message.
    let ciphertext1 = enc_stream.push(msg1.as_bytes(), None, Tag::Message).unwrap();
    
    // encrypt second message, tagging it as push.
    let ciphertext2 = enc_stream.push(msg2.as_bytes(), None, Tag::Push).unwrap();
    
    // encrypt third message, tagging it as final.
    let ciphertext3 = enc_stream.push(msg3.as_bytes(), None, Tag::Final).unwrap();
    
    // initialize decrypt secret stream
    let mut dec_stream = Stream::init_pull(&header, &key).unwrap();
    
    // decrypt first message.
    assert!(!dec_stream.is_finalized());
    let (decrypted1, tag1) = dec_stream.pull(&ciphertext1, None).unwrap();
    assert_eq!(tag1, Tag::Message);
    assert_eq!(msg1.as_bytes(), &decrypted1[..]);
    
    // decrypt second message.
    assert!(!dec_stream.is_finalized());
    let (decrypted2, tag2) = dec_stream.pull(&ciphertext2, None).unwrap();
    assert_eq!(tag2, Tag::Push);
    assert_eq!(msg2.as_bytes(), &decrypted2[..]);
    
    // decrypt last message.
    assert!(!dec_stream.is_finalized());
    let (decrypted3, tag3) = dec_stream.pull(&ciphertext3, None).unwrap();
    assert_eq!(tag3, Tag::Final);
    assert_eq!(msg3.as_bytes(), &decrypted3[..]);
    
    // dec_stream is now finalized.
    assert!(dec_stream.is_finalized());
}


fn main() {

    let _ = sodiumoxide::init();

    use_crypto_aed();
    use_crypto_auth_simple();
    use_crypto_auth_streaming();
    use_crypto_kdf();
    use_box_seal();
    use_crypto_hash();
    use_crypto_kx();
    use_crypto_sign();
    use_crypto_onetimeauth();
    use_crypto_pwhash_verify();
    use_crypto_scalarmult();
    use_crypto_secretbox();
    use_crytp_stream();
    use_crypto_secretstream();
}
