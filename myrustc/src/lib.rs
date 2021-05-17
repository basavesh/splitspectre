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
fn agent_client_fn_return(scope: &mut Scope, fn_name: &str, request: &str, ret: &str) {
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

pub fn gen_agent_client(my_visitor: &CustomItemVisitor) {
    let mut scope = Scope::new();

    scope.import("splitspectre", "*");
    scope.new_module("splitspectre").vis("pub").push_raw("tonic::include_proto!(\"splitspectre\");");

    for (_k, v) in my_visitor.fn_calls.iter() {
        let fn_name = v.segments.last().unwrap().ident.name.to_ident_string();
        let fn_name_cc = fn_name.to_camel_case();

        println!("Old: The function arguments are {:#?} ", v.fn_sig.inputs());
        // transform arguments
        for ty in v.fn_sig.inputs().iter() {
            let ty_string = ty.to_string();
            let ty_string = ty_string.replace("&[secret_integers::U8]", "u64");
            let ty_string = ty_string.replace("std::vec::Vec<secret_integers::U8>", "u64");
            println!("New: The argument type is {:#?}", ty_string);
        }
        println!("Old: The function return type is {:#?} ", v.fn_sig.output());
        let fn_ret = v.fn_sig.output().to_string();
        let fn_ret = fn_ret.replace("[secret_integers::U8]", "u64");
        let fn_ret = fn_ret.replace("std::vec::Vec<secret_integers::U8>", "u64");
        println!("New: The function return type is {:#?} ", fn_ret);
        // println!("The function name is {:?}", fn_name);
        agent_client_fn_return(&mut scope, fn_name.as_str(), "GetSecretKeyRequest {}", fn_ret.as_str());
    }
    // agent_client_fn_return(&mut scope, "EncryptRequest { arg1: arg1.to_vec(), keyid: *arg2,}", "encrypt", "Vec<u8>");
    // agent_client_fn_return(&mut scope, "DecryptRequest { arg1: arg1.to_vec(), keyid: *arg2,}", "decrypt", "Vec<u8>");

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

    agent_server_fn_return(imp,"get_secret_key", "GetSecretKeyRequest", "GetSecretKeyResponse");
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

pub fn gen_agent_server() {
    let mut scope = Scope::new();
    agent_server_imports_and_modules(&mut scope);
    agent_server_classify_declassify(&mut scope);
    agent_server_impl(&mut scope);

    println!("{}", scope.to_string());
}