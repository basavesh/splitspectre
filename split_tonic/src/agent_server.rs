//#![allow(dead_code)]
use tonic::*;
use splitspectre::*;
use secret_integers::*;
use std::sync::*;
use std::collections::HashMap;
use simple::*;

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
    keys_map: Arc<Mutex<HashMap<u64, Vec<U8>>>>,
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
        // get a read lock immediately and get keyid
        // TODO need to supply some extra information to find that.

        // Only if the return type is secret.
        // First, check if the result is already stored in the HashMap

        // XXXXXXX Do we want to de-duplicate the data here
        // Idea 1: more constant time comparison
        // Idea 2: don't do this, just create a new (key, val)
        // Idea 3: use another hashmap
        let call_result = get_secret_key();

        if let Ok(mut write_guard) = self.keys_map.lock() {
            // time to create a new keyid
            let mut num = self.counter.lock().unwrap();
            *num += 1;
            write_guard.insert(*num, call_result);
            let response = GetSecretKeyResponse {
                result: Some(SecretId{keyid: *num}),
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
        if let Ok(read_guard) = self.keys_map.lock() {
            let request = request.into_inner();

            if read_guard.contains_key(&request.arg2.as_ref().unwrap().keyid) {
                let sk = &read_guard[&request.arg2.as_ref().unwrap().keyid];
                let result = encrypt(&request.arg1, &sk);
                let response = EncryptResponse {
                    result,
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
        if let Ok(read_guard) = self.keys_map.lock() {
            let request = request.into_inner();

            if read_guard.contains_key(&request.arg2.as_ref().unwrap().keyid) {
                let sk = &read_guard[&request.arg2.as_ref().unwrap().keyid];
                let result = decrypt(&request.arg1, &sk);
                let response = DecryptResponse {
                    result,
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
        keys_map: Arc::new(Mutex::new(HashMap::new())),
        counter: Arc::new(Mutex::new(0)),
    };
    transport::Server::builder()
        .add_service(agent_server::AgentServer::new(agent))
        .serve(addr).await?;
    Ok(())
}