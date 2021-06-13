use tonic::*;
use splitspectre::*;
use secret_integers::*;
use std::sync::*;
use std::collections::HashMap;
use agent_server_lib::*;

pub mod splitspectre {
    tonic::include_proto!("splitspectre");
}

pub mod agent_server_lib;

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
    keys_map: Arc<Mutex<HashMap<u64, Vec<U8>>>>,
    counter: Arc<Mutex<u64>>,
}

#[tonic::async_trait]
impl agent_server::Agent for MyAgent {
    async fn classify_u8s(&self, request: Request<ClassifyU8sRequest>) -> Result<Response<ClassifyU8sResponse>, Status> {
        let request = request.into_inner();
        let call_result = classify_u8s(&request.arg1, );
        if let Ok(mut lock_guard) = self.keys_map.lock() {
            let mut num = self.counter.lock().unwrap();
            *num += 1;
            lock_guard.insert(*num, call_result);
            let response = ClassifyU8sResponse { result: Some(SecretId{ keyid: *num})};
            return Ok(Response::new(response));
        }
        Err(tonic::Status::unimplemented("Could not obtain lock"))
    }

    async fn chacha20_encrypt(&self, request: Request<Chacha20EncryptRequest>) -> Result<Response<Chacha20EncryptResponse> , Status> {
        let request = request.into_inner();
        if let Ok(lock_guard) = self.keys_map.lock() {
            if lock_guard.contains_key(&request.arg1.as_ref().unwrap().keyid) {
                let sk = &lock_guard[&request.arg1.as_ref().unwrap().keyid];
                let result = chacha20_encrypt(&sk, request.arg2, &request.arg3, &request.arg4, );
                let response = Chacha20EncryptResponse { result, };
                return Ok(Response::new(response));
            }
            return Err(tonic::Status::unimplemented("No corresponding secret for the key provided"));
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