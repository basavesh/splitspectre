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


// Server case is little complicated
fn agent_server_impl () {
    let mut scope = Scope::new();
    scope.import("tonic", "Request");
    scope.import("tonic", "Response");
    scope.import("tonic", "Status");
    scope.import("tonic::transport", "Server");
    let my_struct = scope.new_struct("MyAgent");
    my_struct.field("keys_map", "Arc<RwLock<HashMap<u64, Vec<U8>>>>");
    my_struct.field("counter", "Arc<Mutex<u64>>");
    let imp = scope.new_impl("MyAgent");
    imp.impl_trait("Agent");
    imp.r#macro("#[tonic::async_trait]");

    // Make this a function.
    {
        let my_fn = imp.new_fn("get_secret_key");
        let req_param = "GetSecretKeyRequest";
        let res_param = "GetSecretKeyResponse";
        my_fn
            .set_async(true)
            .arg_ref_self()
            .arg("request", req_param)
            .ret(format!("Result<Response<{}, Status>", res_param));
    }

    // Make this a function
    {
        let my_fn = imp.new_fn("encrypt");
        let req_param = "EncryptRequest";
        let res_param = "EncryptResponse";
        my_fn
            .set_async(true)
            .arg_ref_self()
            .arg("request", req_param)
            .ret(format!("Result<Response<{}, Status>", res_param));
    }

    // Make this a function
    {
        let my_fn = imp.new_fn("encrypt");
        let req_param = "EncryptRequest";
        let res_param = "EncryptResponse";
        my_fn
            .set_async(true)
            .arg_ref_self()
            .arg("request", req_param)
            .ret(format!("Result<Response<{}, Status>", res_param));
    }

    // Make this a different function
    {
        let my_fn = imp.new_fn("main");
        my_fn
            .set_async(true)
            .attr("tokio::main")
            .ret("Result<(), Box<dyn std::error::Error>>")
            .line("let addr = \"127.0.0.1:50051\".parse()?;")
            .line("let agent = MyAgent {")
            .line("    keys_map: Arc::new(RwLock::new(HashMap::new())),")
            .line("    counter: Arc::new(Mutex::new(0)),")
            .line("};")
            .line("Server::builder()")
            .line("    .add_service(AgentServer::new(agent))")
            .line("    .serve(addr).await?;\n")
            .line("Ok(())");


    }

    println!("{}", scope.to_string());
}

fn main() {
    //agent_client_fn_return();
    //println!();
    agent_server_impl();
}
