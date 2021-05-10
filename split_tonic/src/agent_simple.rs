use splitspectre::*;

pub mod splitspectre {
    tonic::include_proto!("splitspectre");
}

pub async fn agent_get_secret_key() -> u64 {
    let mut client = agent_client::AgentClient::connect("http://127.0.0.1:50051").await.unwrap();
    let request = tonic::Request::new(GetSecretKeyRequest {});
    let response = client.get_secret_key(request).await.unwrap().into_inner();
    return response.result;
}

pub async fn agent_encrypt(arg1: &[u8], arg2: &u64) -> Vec<u8> {
    let mut client = agent_client::AgentClient::connect("http://127.0.0.1:50051").await.unwrap();
    let request = tonic::Request::new(EncryptRequest { arg1: arg1.to_vec(), keyid: *arg2,});
    let response = client.encrypt(request).await.unwrap().into_inner();
    return response.result;
}

pub async fn agent_decrypt(arg1: &[u8], arg2: &u64) -> Vec<u8> {
    let mut client = agent_client::AgentClient::connect("http://127.0.0.1:50051").await.unwrap();
    let request = tonic::Request::new(DecryptRequest { arg1: arg1.to_vec(), keyid: *arg2,});
    let response = client.decrypt(request).await.unwrap().into_inner();
    return response.result;
}

