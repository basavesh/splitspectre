#![allow(dead_code)]
use codegen::Scope;

// Need to handle cases later
fn agent_client_fn_return() {
    let mut scope = Scope::new();
    let addr = "http://127.0.0.1:50051";
    let request = "GetSecretKeyRequest {}";
    let fn_name = "get_secret_key";
    scope
        .new_fn("agent_get_secret_key")
        .vis("pub")
        .set_async(true)
        .ret("u64")
        .line(format!("let mut client = AgentClient::connect(\"{}\").await.unwrap();", addr))
        .line(format!("let request = tonic::Request::new({});", request))
        .line(format!("let response = client.{}(request).await.unwrap().into_inner();", fn_name))
        .line(format!("return response.result"));
        println!("{}", scope.to_string());
}

fn main() {
    agent_client_fn_return();
}
