//#![allow(dead_code)]
use tonic::*;
use splitspectre::*;
use secret_integers::*;
use std::sync::*;
use std::collections::HashMap;

pub mod simple;

pub mod splitspectre {
    // the string specified here must match the proto package name
    tonic::include_proto!("splitspectre");
}

/// classify vector of u8s into U8s
#[allow(dead_code)]
fn classify_u8s(v: &[u8]) -> Vec<U8> {
    v.iter().map(|x| U8::classify(*x)).collect()
}

/// declassify vector of U8s into u8s
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
    async fn get_secret_key(
        &self,
        _request: Request<GetSecretKeyRequest>, // No param request
    ) -> Result<Response<GetSecretKeyResponse>, Status> { // Return respone - keyid
        println!("Got a GetSecretKey Request");

        // TODO change the logic of this
        // If the callee function contains secret type in the argument,
        // get a read lock immidiately and get keyid
        // TODO need to supply some extra information to find that.

        // Only if the return type is secret.
        // First, check if the result is already stored in the HashMap
        let call_result = simple::get_secret_key();

        if let Ok(mut write_guard) = self.keys_map.write() {
            for (k, v) in write_guard.iter() {
                // To compare, should declassify secrets
                if declassify_u8s(v) == declassify_u8s(&call_result) {
                    // time to return the result
                    // we already have the key for this
                    let response = GetSecretKeyResponse {
                        result: *k,
                    };
                    return Ok(Response::new(response));
                }
            }
            // time to create a new keyid
            let mut num = self.counter.lock().unwrap();
            *num += 1;
            write_guard.insert(*num, call_result);
            let response = GetSecretKeyResponse {
                result: *num,
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

        // If the callee signature doesn't return a secret type,
        // I should just take the read lock
        if let Ok(read_guard) = self.keys_map.read() {
            let request = request.into_inner();

            if read_guard.contains_key(&request.keyid) {
                let sk = &read_guard[&request.keyid];
                let new_block = simple::encrypt(&request.arg1, &sk);
                let response = EncryptResponse {
                    result: new_block,
                };

                return Ok(Response::new(response));
            }
            return Err(tonic::Status::unimplemented("No corresponding secret for the key provided"));
        }

        Err(tonic::Status::unimplemented("Could not obtain lock"))
    }

    async fn decrypt(
        &self,
        request: Request<DecryptRequest>,
    ) -> Result<Response<DecryptResponse>, Status> {
        println!("Got a decrypt Request");

        // If the callee signature doesn't return a secret type,
        // I should just take the read lock
        if let Ok(read_guard) = self.keys_map.read() {
            let request = request.into_inner();

            if read_guard.contains_key(&request.keyid) {
                let sk = &read_guard[&request.keyid];
                let new_block = simple::decrypt(&request.arg1, &sk);
                let response = DecryptResponse {
                    result: new_block,
                };

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
        keys_map: Arc::new(RwLock::new(HashMap::new())),
        counter: Arc::new(Mutex::new(0)),
    };
    transport::Server::builder()
        .add_service(agent_server::AgentServer::new(agent))
        .serve(addr).await?;
    Ok(())
}