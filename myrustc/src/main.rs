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

use rustc_driver::{catch_with_exit_code, Callbacks, Compilation, RunCompiler};
use rustc_hir::intravisit::{self, NestedVisitorMap, Visitor};
use rustc_errors::{Applicability, DiagnosticBuilder};
use rustc_hir::itemlikevisit::ItemLikeVisitor;
use rustc_hir::{ForeignItem, ImplItem, Item, ItemKind, TraitItem, TyKind};
use rustc_interface::{interface::Compiler, Queries};
use rustc_middle::mir::TerminatorKind;
use rustc_middle::ty::TyCtxt;
use rustc_middle::hir::map::Map;
use rustc_session::Session;
use std::collections::HashMap;

/// Custom Compiler callbacks.format
pub(crate) struct CustomCallbacks;

impl Callbacks for CustomCallbacks {
    fn after_analysis<'tcx>(
        &mut self,
        compiler: &Compiler,
        queries: &'tcx Queries<'tcx>,
    ) -> Compilation {
        // println!("Hello from rustc after analysis");
        compiler.session().abort_if_errors();

        let crate_name = queries.crate_name().unwrap().peek();
        let secret_crate;
        if *crate_name == "secret_integers" {
            secret_crate = true;
        } else if *crate_name == "secret_integers_usage" {
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
                                            };
            tcx.hir().krate().visit_all_item_likes(&mut item_visitor);

            for bid in item_visitor.body_ids {
                // println!("Body ID: {:#?}", bid);
                let mut deep_visitor = DeepVisitor {
                                                    tcx,
                                                    sess: tcx.sess,
                                                    secret_crate,
                                                    fn_defs: HashMap::new(),
                                                    fn_calls: HashMap::new(),
                                                    };
                //tcx.hir().krate().visit_all_item_likes(&mut deep_visitor.as_deep_visitor());
                deep_visitor.visit_nested_body(bid);

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

struct CustomItemVisitor<'tcx> {
    tcx: TyCtxt<'tcx>,
    sess: &'tcx Session,  // not sure if I need to use this.
    secret_crate: bool,
    inside_secret_fn: bool,
    body_ids: Vec<rustc_hir::BodyId>,
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
            } else {
                // walk this body and check function / method calls.
                self.body_ids.push(body_id);
            }
        }
    }
    fn visit_trait_item(&mut self, _trait_item: &'hir TraitItem<'hir>) {}
    fn visit_impl_item(&mut self, _impl_item: &'hir ImplItem<'hir>) {}
    fn visit_foreign_item(&mut self, _foreign_item: &'hir ForeignItem<'hir>) {}
}

struct DeepVisitor<'tcx> {
    tcx: TyCtxt<'tcx>,
    sess: &'tcx Session,
    secret_crate: bool,
    fn_defs: HashMap<rustc_span::def_id::DefId , FnDef>,
    fn_calls: HashMap<rustc_span::def_id::DefId , Vec<FnCall<'tcx>>>,
}

#[derive(Debug)]
struct FnDef {
    ident: rustc_span::symbol::Ident,
    span: rustc_span::Span,
}

#[derive(Debug)]
struct FnCall<'tcx>{
    span: rustc_span::Span,
    segments: &'tcx [rustc_hir::PathSegment<'tcx>],
}

impl<'tcx> intravisit::Visitor<'tcx> for DeepVisitor<'tcx> {
    type Map = Map<'tcx>;

    fn nested_visit_map(&mut self) -> intravisit::NestedVisitorMap<Self::Map> {
        intravisit::NestedVisitorMap::OnlyBodies(self.tcx.hir())
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
                if let rustc_hir::def::Res::Def(_def_kind, def_id) = fn_path.res {
                    let fn_call_sig = self.tcx.fn_sig(def_id);
                    let fn_call_str = format!("{}", fn_call_sig);
                    if fn_call_str.contains("secret_integers::U8") {
                        let def_id_clone = def_id.clone();
                        if self.fn_calls.contains_key(&def_id_clone) {
                            self.fn_calls.get_mut(&def_id_clone)
                                            .unwrap()
                                            .push(FnCall{
                                                    segments: fn_path.segments,
                                                    span: expr.span,
                                            });
                        } else {
                            self.fn_calls.insert(def_id_clone,
                                                 vec!(FnCall{
                                                        segments: fn_path.segments,
                                                        span: expr.span
                                                 }));
                        }

                        let mut diag = self.sess.struct_span_warn(expr.span, "Test this warning message");
                        let snip = self.sess.source_map().span_to_snippet(expr.span).unwrap();
                        diag.span_suggestion(expr.span, "try using agent_call here", format!("agent_{}", snip), Applicability::MachineApplicable);
                        diag.emit();
                        return; // Don't go deeper inside this.
                    }
                }
            }
            return;
        } else {
            intravisit::walk_expr(self, expr);
        }
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