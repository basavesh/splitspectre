use std::io;
use agent_simple::*;
pub mod agent_simple;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>  {
    let secret_key: u64 = agent_get_secret_key().await;
    let mut buffer: String = String::new();
    println!("Please input a 8 byte message");
    let _ = io::stdin().read_line(&mut buffer);
    let mut message = [0u8; 8];
    for i in 0..8 {
        if i < buffer.len() {
            message[i] = buffer.as_bytes()[i];
        }
    }
    let cipher_text: Vec<u8> = agent_encrypt(&message, &secret_key).await;
    let text: Vec<u8> = agent_decrypt(&cipher_text, &secret_key).await;
    assert!(message == &text[..]);
    println!("message: {:?}", message);
    println!("ciphertext: {:?}", cipher_text);

    Ok(())
}
