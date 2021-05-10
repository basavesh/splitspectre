#![allow(dead_code)]
use codegen::*;

// Need to handle cases later
fn agent_client_fn_return(scope: &mut Scope, request: &str, fn_name: &str, ret: &str) {
    let addr = "http://127.0.0.1:50051";

    scope
        .new_fn(format!("agent_{}",fn_name).as_str())
        .vis("pub")
        .set_async(true)
        .ret(ret)
        .line(format!("let mut client = agent_client::AgentClient::connect(\"{}\").await.unwrap();", addr))
        .line(format!("let request = tonic::Request::new({});", request))
        .line(format!("let response = client.{}(request).await.unwrap().into_inner();", fn_name))
        .line(format!("return response.result;"));
}

fn gen_agent_client () {

    let mut scope = Scope::new();

    scope.import("splitspectre", "*");
    scope.new_module("splitspectre").vis("pub").push_raw("tonic::include_proto!(\"splitspectre\");");

    agent_client_fn_return(&mut scope, "GetSecretKeyRequest {}", "get_secret_key", "u64");
    agent_client_fn_return(&mut scope, "EncryptRequest { arg1: arg1.to_vec(), keyid: *arg2,}", "encrypt", "Vec<u8>");
    agent_client_fn_return(&mut scope, "DecryptRequest { arg1: arg1.to_vec(), keyid: *arg2,}", "decrypt", "Vec<u8>");

    println!("{}", scope.to_string());
}

fn agent_server_fn_return(imp: &mut Impl, fn_name: &str, request: &str, response: &str) {
    imp
        .new_fn(fn_name)
        .set_async(true)
        .arg_ref_self()
        .arg("request", request)
        .ret(format!("Result<Response<{}, Status>", response));
}

// Server case is little complicated
fn agent_server_impl(scope: &mut Scope) {
    // MyAgent Struct
    scope
        .new_struct("MyAgent")
        .derive("Debug")
        .derive("Default")
        .field("keys_map", "Arc<RwLock<HashMap<u64, Vec<U8>>>>")
        .field("counter", "Arc<Mutex<u64>>");
    let imp = scope.new_impl("MyAgent");
    imp.impl_trait("agent_server::Agent");
    imp.r#macro("#[tonic::async_trait]");

    agent_server_fn_return(imp,"get_secret_key", "GetSecretKey", "GetSecretKeyResponse");
    agent_server_fn_return(imp, "encrypt", "EncryptRequest", "EncryptResponse");
    agent_server_fn_return(imp, "decrypt", "DecryptRequest", "DecryptResponse");

}

fn agent_server_imports_and_modules(scope: &mut Scope) {
    scope.import("tonic", "*");
    scope.import("splitspectre", "*");
    scope.import("secret_integers", "*");
    scope.import("std::sync", "*");
    scope.import("std::collections", "HashMap");
    scope.new_module("splitspectre").vis("pub").push_raw("tonic::include_proto!(\"splitspectre\");");

    // This is something not standard
    scope.raw("pub mod simple;");
}

fn agent_server_classify_declassify(scope: &mut Scope) {
    // classify
    scope
        .new_fn("classify_u8s")
        .attr("allow(dead_code)")
        .arg("v", "&[u8]")
        .ret("Vec<U8>")
        .line("v.iter().map(|x| U8::classify(*x)).collect()");
    scope
        .new_fn("declassify_u8s")
        .attr("allow(dead_code)")
        .arg("v", "&[U8]")
        .ret("Vec<u8>")
        .line("v.iter().map(|x| U8::declassify(*x)).collect()");
}

fn gen_agent_server() {
    let mut scope = Scope::new();
    agent_server_imports_and_modules(&mut scope);
    agent_server_classify_declassify(&mut scope);
    agent_server_impl(&mut scope);

    println!("{}", scope.to_string());
}

fn main() {
    // gen_agent_client();
    //println!();
    gen_agent_server();
    //gen_agent_server_impl();
}
