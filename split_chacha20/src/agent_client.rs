use splitspectre::*;

pub mod splitspectre {
    tonic::include_proto!("splitspectre");
}

pub async fn agent_classify_u8s(arg1: &[u8]) -> u64 {
    let mut client = agent_client::AgentClient::connect("http://127.0.0.1:50051").await.unwrap();
    let request = tonic::Request::new(ClassifyU8sRequest { arg1 : arg1.to_vec(),});
    let response = client.classify_u8s(request).await.unwrap().into_inner();
    return response.result.unwrap().keyid;
}

pub async fn agent_chacha20_encrypt() -> u64 {
    let mut client = agent_client::AgentClient::connect("http://127.0.0.1:50051").await.unwrap();
    let request = tonic::Request::new(Chacha20EncryptRequest {});
    let response = client.chacha20_encrypt(request).await.unwrap().into_inner();
    return response.result.unwrap().keyid;
}