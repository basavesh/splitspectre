use splitspectre::agent_client::AgentClient;
use splitspectre::*;

pub mod splitspectre {
    tonic::include_proto!("splitspectre");
}

pub async fn agent_get_secret_key() -> u64 {
    let mut client = AgentClient::connect("http://127.0.0.1:50051").await.unwrap();
    let request = tonic::Request::new(GetSecretKeyRequest {});
    let response = client.get_secret_key(request).await.unwrap().into_inner();
    return response.keyid;
}

pub async fn agent_encrypt(msg: &[u8], sk: &u64) -> Vec<u8> {
    let mut client = AgentClient::connect("http://127.0.0.1:50051").await.unwrap();
    let request = tonic::Request::new(EncryptRequest {
        msg: msg.to_vec(),
        keyid: *sk,
    });
    let response = client.encrypt(request).await.unwrap().into_inner();
    return response.cipher;
}

pub async fn agent_decrypt(cipher: &[u8], sk: &u64) -> Vec<u8> {
    let mut client = AgentClient::connect("http://127.0.0.1:50051").await.unwrap();
    let request = tonic::Request::new(DecryptRequest {
        cipher: cipher.to_vec(),
        keyid: *sk,
    });
    let response = client.decrypt(request).await.unwrap().into_inner();
    return response.msg;
}

