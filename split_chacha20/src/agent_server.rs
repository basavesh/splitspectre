use tonic::*;
use splitspectre::*;
use secret_integers::*;
use std::sync::*;
use std::collections::HashMap;

pub mod splitspectre {
    tonic::include_proto!("splitspectre");
}

pub mod simple;

#[allow(dead_code)]
fn classify_u8s(v: &[u8]) -> Vec<U8> {
    v.iter().map(|x| U8::classify(*x)).collect()
}

#[allow(dead_code)]
fn declassify_u8s(v: &[U8]) -> Vec<u8> {
    v.iter().map(|x| U8::declassify(*x)).collect()
}

#[derive(Debug, Default)]
struct MyAgent {
    keys_map: Arc<RwLock<HashMap<u64, Vec<U8>>>>,
    counter: Arc<Mutex<u64>>,
}

#[tonic::async_trait]
impl agent_server::Agent for MyAgent {
    async fn classify_u8s(&self, request: ClassifyU8sRequest) -> Result<Response<ClassifyU8sResponse, Status> {
        let request = request.into_inner();
        let call_result = classify_u8s(&request.arg1, );
        if let Ok(mut lock_guard) = self.keys_map.lock() {
            let mut num = self.counter.lock().unwrap();
            *num += 1;
            lock_guard.insert(*num, call_result);
            let response = ClassifyU8s { result: Some(SecretId{ keyid: *num})
            return Ok(Response::new(response));
        }
        Err(tonic::Status::unimplemented("Could not obtain lock"))
    }

    async fn chacha20_encrypt(&self, request: Chacha20EncryptRequest) -> Result<Response<Chacha20EncryptResponse, Status> {
        let request = request.into_inner();
        let call_result = chacha20_encrypt(&request.arg1, request.arg2, &request.arg3, &request.arg4, );
        if let Ok(mut lock_guard) = self.keys_map.lock() {
            let mut num = self.counter.lock().unwrap();
            *num += 1;
            lock_guard.insert(*num, call_result);
            let response = Chacha20Encrypt { result: Some(SecretId{ keyid: *num})
            return Ok(Response::new(response));
        }
        Err(tonic::Status::unimplemented("Could not obtain lock"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse()?;
    let agent = MyAgent {
        keys_map: Arc::new(Mutex::new(HashMap::new())),
        counter: Arc::new(Mutex::new(0)),
    };
    transport::Server::builder()
        .add_service(agent_server::AgentServer::new(agent))
        .serve(addr).await?;
    Ok(())
}