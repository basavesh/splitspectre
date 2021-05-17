#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_attributes)]

// Copied from main
#![feature(rustc_private)]
// #![feature(in_band_lifetimes)]

extern crate rustc_driver;
extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_span;
extern crate rustc_session;
extern crate rustc_errors;
extern crate if_chain;
extern crate rustc_hir_pretty;

use rustc_driver::{catch_with_exit_code, Callbacks, Compilation, RunCompiler};
use rustc_hir::intravisit::{self, NestedVisitorMap, Visitor};
use rustc_errors::{Applicability, DiagnosticBuilder};
use rustc_hir::itemlikevisit::ItemLikeVisitor;
use rustc_hir::{ForeignItem, ImplItem, Item, ItemKind, TraitItem, TyKind, Stmt};
use rustc_interface::{interface::Compiler, Queries};
use rustc_middle::mir::TerminatorKind;
use rustc_middle::ty::TyCtxt;
use rustc_middle::hir::map::Map;
use rustc_session::Session;
use std::collections::HashMap;
use rustc_hir_pretty::ty_to_string;

use codegen::*;
use heck::CamelCase;



pub struct CustomItemVisitor<'tcx> {
    pub tcx: TyCtxt<'tcx>,
    pub sess: &'tcx Session,  // not sure if I need to use this.
    pub secret_crate: bool,
    pub inside_secret_fn: bool,
    pub body_ids: Vec<rustc_hir::BodyId>,
    pub fn_defs: HashMap<rustc_span::def_id::DefId , FnDef>,
    pub fn_calls: HashMap<rustc_span::def_id::DefId , FnCall<'tcx>>,
}

#[derive(Debug)]
pub struct FnDef {
    pub ident: rustc_span::symbol::Ident,
    pub span: rustc_span::Span,
}

#[derive(Debug)]
pub struct FnCall<'tcx>{
    pub spans: Vec<rustc_span::Span>,
    pub segments: &'tcx [rustc_hir::PathSegment<'tcx>],
    pub name: String,
    pub fn_sig: rustc_middle::ty::FnSig<'tcx>,
}


// Need to handle cases later
fn agent_client_fn_return(scope: &mut Scope, fn_name: &str, fn_args: Vec<(String, String)>, request: &str, ret: &str, ret_secret: bool) {
    let addr = "http://127.0.0.1:50051";

    let my_fn = scope
        .new_fn(format!("agent_{}",fn_name).as_str())
        .vis("pub")
        .set_async(true)
        .ret(ret)
        .line(format!("let mut client = agent_client::AgentClient::connect(\"{}\").await.unwrap();", addr))
        .line(format!("let request = tonic::Request::new({});", request))
        .line(format!("let response = client.{}(request).await.unwrap().into_inner();", fn_name));

    for arg in fn_args.iter() {
        my_fn.arg(&arg.0, &arg.1);
    }

    if ret_secret {
        my_fn.line(format!("return response.result.unwrap().keyid;"));
    } else {
        my_fn.line(format!("return response.result;"));
    }

}

pub fn gen_agent_client(my_visitor: &CustomItemVisitor) {
    let mut scope = Scope::new();

    scope.import("splitspectre", "*");
    scope.new_module("splitspectre").vis("pub").push_raw("tonic::include_proto!(\"splitspectre\");");

    for (_k, v) in my_visitor.fn_calls.iter() {
        let fn_name = v.segments.last().unwrap().ident.name.to_ident_string();
        // let fn_name_cc = fn_name.to_camel_case();

        // transform arguments
        let mut fn_args = Vec::new();
        let mut request: String = format!("{}Request {{", fn_name.to_camel_case());
        for (i, ty) in v.fn_sig.inputs().iter().enumerate() {
            if let rustc_middle::ty::TyKind::Ref(_, ref_ty, _) = ty.kind() {
                if let rustc_middle::ty::TyKind::Slice(slice_ty) = ref_ty.kind() {
                    if slice_ty.to_string() == "secret_integers::U8" {
                        fn_args.push((format!("arg{}", i+1), "&u64".to_string()));
                        request.push_str(format!(" arg{} : Some(SecretId{{ keyid: *arg{},}}),", i+1, i +1).as_str());
                    } else {
                        fn_args.push((format!("arg{}", i+1), ty.to_string()));
                        request.push_str(format!(" arg{} : arg{}.to_vec(),", i+1, i +1).as_str());
                    }
                }
                // TODO: handle other cases
            }
            // TODO: handle other cases
        }
        request.push('}');

        // transform return
        let mut fn_ret = v.fn_sig.output().to_string();
        let ret_secret = fn_ret.contains("secret_integers::U8");
        if ret_secret {
            fn_ret = fn_ret.replace("[secret_integers::U8]", "u64");
            fn_ret = fn_ret.replace("std::vec::Vec<secret_integers::U8>", "u64");
        }
        agent_client_fn_return(&mut scope, &fn_name, fn_args, &request, &fn_ret, ret_secret);
    }
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
fn agent_server_impl(scope: &mut Scope, my_visitor: &CustomItemVisitor) {
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

    for (_k, v) in my_visitor.fn_calls.iter() {
        let fn_name = v.segments.last().unwrap().ident.name.to_ident_string();
        let request = format!("{}Request", fn_name.to_camel_case());
        let respone = format!("{}Response", fn_name.to_camel_case());

        agent_server_fn_return(imp, &fn_name, &request, &respone);
    }
}

fn agent_server_imports_and_modules(scope: &mut Scope, ) {
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

fn agent_server_main(scope: &mut Scope){
    scope
        .new_fn("main")
        .set_async(true)
        .attr("tokio::main")
        .ret("Result<(), Box<dyn std::error::Error>>")
        .line("let addr = \"127.0.0.1:50051\".parse()?;")
        .line("let agent = MyAgent {")
        .line("    keys_map: Arc::new(Mutex::new(HashMap::new())),")
        .line("    counter: Arc::new(Mutex::new(0)),")
        .line("};")
        .line("transport::Server::builder()")
        .line("    .add_service(agent_server::AgentServer::new(agent))")
        .line("    .serve(addr).await?;")
        .line("Ok(())");
}

pub fn gen_agent_server(my_visitor: &CustomItemVisitor) {
    let mut scope = Scope::new();
    agent_server_imports_and_modules(&mut scope);
    agent_server_classify_declassify(&mut scope);
    agent_server_impl(&mut scope, my_visitor);
    agent_server_main(&mut scope);

    println!("{}", scope.to_string());
}