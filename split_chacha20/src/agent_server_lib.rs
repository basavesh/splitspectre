pub fn classify_u8s(v: &[u8]) -> Vec<U8> {
    v.iter().map(|x| U8::classify(*x)).collect()
}
fn chacha20_init(k: &Key, counter: U32, nonce: &Nonce) -> State {
    let mut st = [U32::classify(0u32); 16];
    st[0..4].copy_from_slice(&classify_u32s(&CONSTANTS));
    st[4..12].copy_from_slice(U32::from_bytes_le(k).as_slice());
    st[12] = counter;
    st[13..16].copy_from_slice(U32::from_bytes_le(nonce).as_slice());
    st
}
fn chacha20(k: &Key, counter: U32, nonce: &Nonce) -> State {
    let mut st = chacha20_init(k, counter, nonce);
    chacha20_core(&mut st);
    st
}
fn chacha20_block(k: &Key, counter: U32, nonce: &Nonce) -> Block {
    let st = chacha20(k, counter, nonce);
    let mut block = [U8::classify(0u8); BLOCK_SIZE];
    block.copy_from_slice(U32::to_bytes_le(&st).as_slice());
    block
}
fn xor_block(block: &Block, key_block: &Block) -> Block {
    let mut v_out = [Default::default(); BLOCK_SIZE];
    for i in 0..BLOCK_SIZE {
        v_out[i] = block[i] ^ key_block[i];
    }
    let mut out = [Default::default(); BLOCK_SIZE];
    out.copy_from_slice(&v_out);
    out
}
fn chacha20_counter_mode(key: &Key, counter: U32, nonce: &Nonce, msg: &Vec<U8>) -> Vec<U8> {
    let mut blocks: Vec<[U8; BLOCK_SIZE]> = msg
        .chunks(BLOCK_SIZE)
        .map(|block| {
            let mut new_block = [U8::zero(); BLOCK_SIZE];
            new_block[0..block.len()].copy_from_slice(block);
            new_block
        })
        .collect();
    let nb_blocks = blocks.len();
    let mut key_block: [U8; BLOCK_SIZE];
    let mut ctr = counter;
    for i in 0..blocks.len() - 1 {
        key_block = chacha20_block(key, ctr, nonce);
        blocks[i] = xor_block(&blocks[i], &key_block);
        ctr += U32::one();
    }
    let last = &mut blocks[nb_blocks - 1];
    key_block = chacha20_block(key, ctr, nonce);
    *last = xor_block(last, &key_block);
    blocks
        .iter()
        .map(|block| block.to_vec())
        .flatten()
        .take(msg.len())
        .collect()
}
pub fn chacha20_encrypt(key: &Key, counter: U32, nonce: &Nonce, msg: &Vec<U8>) -> Vec<U8> {
    chacha20_counter_mode(key, counter, nonce, msg)
}
pub fn chacha20_decrypt(key: &Key, counter: U32, nonce: &Nonce, msg: &Vec<U8>) -> Vec<U8> {
    chacha20_counter_mode(key, counter, nonce, msg)
}