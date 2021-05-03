use tonic::{transport::Server, Request, Response, Status};
use splitspectre::agent_server::{Agent, AgentServer};
use splitspectre::*;

use secret_integers::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;

pub mod simple;

pub mod splitspectre {
    // the string specified here must match the proto package name
    tonic::include_proto!("splitspectre");
}

#[derive(Debug, Default)]
pub struct MyAgent {
    keys_map: Arc<RwLock<HashMap<u64, Vec<U8>>>>,
    counter: Arc<Mutex<u64>>,
}

#[tonic::async_trait]
impl Agent for MyAgent {
    async fn get_secret_key(
        &self,
        _request: Request<GetSecretKeyRequest>, // No param request
    ) -> Result<Response<GetSecretKeyResponse>, Status> { // Return respone - keyid
        println!("Got a GetSecretKey Request");

        if let Ok(mut write_guard) = self.keys_map.write() {
            let mut num = self.counter.lock().unwrap();
            *num += 1;
            write_guard.insert(*num, simple::get_secret_key());
            let response = GetSecretKeyResponse {
                keyid: *num,
            };

            return Ok(Response::new(response));
        }

        Err(tonic::Status::unimplemented("Could not obtain lock"))
    }

    async fn encrypt(
        &self,
        request: Request<EncryptRequest>,
    ) -> Result<Response<EncryptResponse>, Status> {
        println!("Got an encrypt Request");

        if let Ok(read_guard) = self.keys_map.read() {
            let request = request.into_inner();

            if read_guard.contains_key(&request.keyid) {
                let sk = &read_guard[&request.keyid];
                let new_block = simple::encrypt(&request.msg, &sk);
                let response = EncryptResponse {
                    cipher: new_block,
                };

                return Ok(Response::new(response));
            }
            return Err(tonic::Status::unimplemented("Could not obtain lock"));
        }

        Err(tonic::Status::unimplemented("Could not obtain lock"))
    }

    async fn decrypt(
        &self,
        request: Request<DecryptRequest>,
    ) -> Result<Response<DecryptResponse>, Status> {
        println!("Got a decrypt Request");

        if let Ok(read_guard) = self.keys_map.read() {
            let request = request.into_inner();

            if read_guard.contains_key(&request.keyid) {
                let sk = &read_guard[&request.keyid];
                let new_block = simple::decrypt(&request.cipher, &sk);
                let response = DecryptResponse {
                    msg: new_block,
                };

                return Ok(Response::new(response));
            }
            return Err(tonic::Status::unimplemented("Could not obtain lock"));
        }

        Err(tonic::Status::unimplemented("Could not obtain lock"))
    }
}

#[tokio::main]
async fn main () -> Result<(), Box<dyn std::error::Error>>{

    let addr = "127.0.0.1:50051".parse()?;
    let agent = MyAgent {
        keys_map: Arc::new(RwLock::new(HashMap::new())),
        counter: Arc::new(Mutex::new(0)),
    };

    Server::builder()
        .add_service(AgentServer::new(agent))
        .serve(addr)
        .await?;

    Ok(())
}