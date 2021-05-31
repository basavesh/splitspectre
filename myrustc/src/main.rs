// This is required to be able to use the rustc crates
#![feature(rustc_private)]
#![feature(in_band_lifetimes)]

#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_imports)]

// This is only required for the `rustc_*` crates.
// Regular dependencies can be used without it.
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

use heck::CamelCase;

// Write the output into files
use std::env;
use std::fs::File;
use std::fs::OpenOptions;

pub mod lib;
use lib::*;
use std::io::prelude::*;

/// Custom Compiler callbacks
pub(crate) struct CustomCallbacks;

impl Callbacks for CustomCallbacks {

    fn config(&mut self, config: &mut rustc_interface::interface::Config) {
        config.opts.debugging_opts.save_analysis = true;
    }

    fn after_analysis<'tcx>(
        &mut self,
        compiler: &Compiler,
        queries: &'tcx Queries<'tcx>,
    ) -> Compilation {
        compiler.session().abort_if_errors();

        let crate_name = queries.crate_name().unwrap().peek();
        let secret_crate;
        if *crate_name == "secret_integers" {
            secret_crate = true;
        } else if *crate_name == "secret_integers_usage" || *crate_name == "chacha20" {
            secret_crate = false;
        } else {
            return Compilation::Continue;
        }
        queries.global_ctxt().unwrap().peek_mut().enter(|tcx| {
            let mut item_visitor = CustomItemVisitor { tcx,
                                                sess: tcx.sess,
                                                secret_crate,
                                                inside_secret_fn: false,
                                                body_ids: Vec::new(),
                                                fn_defs: Vec::<String>::new(),
                                                fn_calls: HashMap::new(),
                                            };
            tcx.hir().krate().visit_all_item_likes(&mut item_visitor);
            if *crate_name == "secret_integers_usage" || *crate_name == "chacha20" {
                lib::gen_agent_client(&item_visitor); // This works fine
                lib::gen_agent_server(&item_visitor); // This works fine
                lib::gen_agent_sever_lib(&item_visitor); // need to take of the imports
                lib::gen_agent_proto(&item_visitor);
                lib::gen_agent_build();
                lib::gen_agent_cargo();
            }
        });
        Compilation::Continue
    }
}

fn generated_code(sess: &'tcx Session, span: rustc_span::Span) -> bool {
    if span.from_expansion() || span.is_dummy() {
        return true;
    }
    // code from rust/compiler/rustc_save_analysis/src/span_utils.rs
    !sess.source_map().lookup_char_pos(span.lo()).file.is_real_file()
}

impl<'hir, 'tcx> ItemLikeVisitor<'hir> for CustomItemVisitor<'tcx> {
    fn visit_item(&mut self, item: &'hir Item<'hir>) {
        if let ItemKind::Fn(_, _, body_id) = item.kind {
            let def_id = self.tcx.hir()
                            .local_def_id(item.hir_id()).to_def_id();
            let fn_call_sig = self.tcx.fn_sig(def_id);
            let fn_call_str = fn_call_sig.to_string();
            if fn_call_str.contains("secret_integers::U8") {
                // This function should be moved to `trusted` process.
                println!("Move fn: {} to trusted process", item.ident.name.to_ident_string());
                let snip = self.sess.source_map().span_to_snippet(item.span).unwrap();
                // println!("{}\n", snip);
                self.fn_defs.push(snip);
            } else {
                self.visit_nested_body(body_id);
            }
        }
    }
    fn visit_trait_item(&mut self, _trait_item: &'hir TraitItem<'hir>) {}
    fn visit_impl_item(&mut self, _impl_item: &'hir ImplItem<'hir>) {}
    fn visit_foreign_item(&mut self, _foreign_item: &'hir ForeignItem<'hir>) {}
}

impl<'tcx> intravisit::Visitor<'tcx> for CustomItemVisitor<'tcx> {
    type Map = Map<'tcx>;

    fn nested_visit_map(&mut self) -> intravisit::NestedVisitorMap<Self::Map> {
        intravisit::NestedVisitorMap::OnlyBodies(self.tcx.hir())
    }

    fn visit_stmt(&mut self, s: &'tcx Stmt<'tcx>) {

        if let rustc_hir::StmtKind::Local(local) = s.kind {
            if let Some(ty_info) = local.ty {
                if ty_to_string(ty_info).contains("secret_integers::U8") {
                    let mut diag = self.sess.struct_span_warn(ty_info.span, "Test this type warning message");
                    diag.span_suggestion(ty_info.span, "try using u64 here", format!("u64"), Applicability::MachineApplicable);
                    diag.emit();
                }
            }
        }
        intravisit::walk_stmt(self, s);
    }

    fn visit_expr(&mut self, expr: &'tcx rustc_hir::Expr<'tcx>) {
        if self.secret_crate {
            return;
        }

        if let rustc_hir::ExprKind::Call(exp, ..) = expr.kind {
            if generated_code(self.sess, expr.span) {
                return;
            }

            if let rustc_hir::ExprKind::Path(rustc_hir::QPath::Resolved(None, fn_path)) = exp.kind {
                let fn_name = fn_path.segments.last().unwrap().ident.name.to_ident_string();
                if let rustc_hir::def::Res::Def(_def_kind, def_id) = fn_path.res {
                    let fn_call_sig = self.tcx.fn_sig(def_id).skip_binder();
                    let fn_call_str = format!("{}", fn_call_sig);
                    if fn_call_str.contains("secret_integers::U8") {
                        let def_id_clone = def_id.clone();
                        if self.fn_calls.contains_key(&def_id_clone) {
                            self.fn_calls.get_mut(&def_id_clone)
                                            .unwrap()
                                            .spans
                                            .push(expr.span);
                        } else {
                            self.fn_calls.insert(def_id_clone,
                                                    FnCall{
                                                        segments: fn_path.segments,
                                                        spans: vec!(expr.span),
                                                        name: fn_name,
                                                        fn_sig: fn_call_sig.clone(),
                                                    }
                                                );
                        }

                        let mut diag = self.sess.struct_span_warn(expr.span, "Test this warning message");
                        let snip = self.sess.source_map().span_to_snippet(expr.span).unwrap();
                        diag.span_suggestion(expr.span, "try using agent_call here", format!("agent_{}", snip), Applicability::MachineApplicable);
                        diag.emit();
                    }
                }
            }
        }

        intravisit::walk_expr(self, expr);
    }
}

// Run the compiler with custom callbacks and return the exit status code.
pub fn run_compiler(args: Vec<String>) -> i32 {
    catch_with_exit_code(move || RunCompiler::new(&args, &mut CustomCallbacks).run())
}

/// Adds the correct --sysroot option.
fn sys_root() -> Vec<String> {
    let home = option_env!("RUSTUP_HOME");
    let toolchain = option_env!("RUSTUP_TOOLCHAIN");
    let sysroot = format!("{}/toolchains/{}", home.unwrap(), toolchain.unwrap());
    vec!["--sysroot".into(), sysroot]
}

fn main() {
    let _ = rustc_driver::catch_fatal_errors(|| {
        // Grab the command line arguments.
        let args: Vec<_> = std::env::args_os().flat_map(|s| s.into_string()).collect();
        let args2 = args.iter()
            .map(|s| (*s).to_string())
            .chain(sys_root().into_iter())
            .collect::<Vec<_>>();

        RunCompiler::new(&args2, &mut CustomCallbacks).run()
    }).map_err(|e| println!("{:?}", e));
}