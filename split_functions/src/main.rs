// This is required to be able to use the rustc crates.
#![feature(rustc_private)]
#![feature(in_band_lifetimes)]

#![allow(non_camel_case_types)]
#![allow(dead_code)]

// This is only required for the `rustc_*` crates. Regular dependencies can be used without it.
extern crate rustc_driver;
extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_span;
extern crate rustc_session;
extern crate rustc_errors;
extern crate if_chain;  // TODO convert some to if_chain!

use rustc_driver::{catch_with_exit_code, Callbacks, Compilation, RunCompiler};
use rustc_hir::intravisit::{self, Visitor};
use rustc_errors::{Applicability, DiagnosticBuilder};
// use rustc_hir::itemlikevisit::ItemLikeVisitor;
// use rustc_hir::{ForeignItem, ImplItem, Item, ItemKind, TraitItem, TyKind};
use rustc_interface::{interface::Compiler, Queries};
// use rustc_middle::mir::TerminatorKind;
use rustc_middle::ty::TyCtxt;
use rustc_middle::hir::map::Map;
use rustc_session::Session;
use std::collections::HashMap;

// Custom Compiler Callbacks
pub(crate) struct CustomCallbacks;

impl Callbacks for CustomCallbacks {

    fn after_analysis<'tcx>(
        &mut self,
        compiler: &Compiler,
        queries: &'tcx Queries<'tcx>
    ) -> Compilation {

        compiler.session().abort_if_errors();
        let crate_name = queries.crate_name().unwrap().peek();

        // let us worry only about this crate
        if *crate_name == "secret_integers_usage" {
            println!("Hello {}", crate_name);
            queries.global_ctxt().unwrap().peek_mut().enter( |tcx|{
                let hir = tcx.hir();
                let krate = hir.krate();
                let mut visitor = CustomVisitor {
                                    tcx, sess: tcx.sess,
                                    secret: false,
                                    fn_defs: HashMap::new(),
                                    fn_calls: HashMap::new(),
                                };
                krate.visit_all_item_likes(&mut visitor.as_deep_visitor()); // can be done in one line. change later.
                //println!("\n\nFn_Defs:\n{:#?}\n", visitor.fn_defs);
                //println!("\n\nFn_Calls:\n{:#?}\n", visitor.fn_calls);
            });
        }
            else if *crate_name == "secret_integers" {
            println!("Hello {}", crate_name);
            queries.global_ctxt().unwrap().peek_mut().enter( |tcx|{
                let hir = tcx.hir();
                let krate = hir.krate();
                let mut visitor = CustomVisitor {
                                    tcx, sess: tcx.sess,
                                    secret: true,
                                    fn_defs: HashMap::new(),
                                    fn_calls: HashMap::new(),
                                };
                krate.visit_all_item_likes(&mut visitor.as_deep_visitor());
            });
        }

        Compilation::Continue
    }

}

struct CustomVisitor<'tcx> {
    tcx: TyCtxt<'tcx>,
    sess: &'tcx Session,
    secret: bool,
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

fn generated_code(sess: &'tcx Session, span: rustc_span::Span) -> bool {
    if span.from_expansion() || span.is_dummy() {
        return true;
    }
    // code from rust/compiler/rustc_save_analysis/src/span_utils.rs
    !sess.source_map().lookup_char_pos(span.lo()).file.is_real_file()
}

impl<'tcx> intravisit::Visitor<'tcx> for CustomVisitor<'tcx> {
    type Map = Map<'tcx>;

    fn nested_visit_map(&mut self) -> intravisit::NestedVisitorMap<Self::Map> {
        intravisit::NestedVisitorMap::OnlyBodies(self.tcx.hir())
    }

    fn visit_expr(&mut self, expr: &'tcx rustc_hir::Expr<'tcx>) {
        if self.secret {
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
                    }
                }
            }
            return;
        }

        intravisit::walk_expr(self, expr);
    }

    // Planning to use this only for "secret_integers" library crate
    fn visit_ty(&mut self, t: &'tcx rustc_hir::Ty<'tcx>) {
        if !self.secret {
            return;
        }

        if generated_code(self.sess, t.span) {
            return;
        }

        if let rustc_hir::TyKind::Path(rustc_hir::QPath::Resolved(None, ty_path)) = t.kind {
            if let rustc_hir::def::Res::PrimTy(prim_ty) = ty_path.res {
                match prim_ty {
                    rustc_hir::PrimTy::Uint(rustc_ast::ast::UintTy::U8) => {
                        // println!("Voila {:#?}", );
                        // TODO this is the Secret_Integers::U8 type
                    }
                    _ => {}
                }
            }
        }
        intravisit::walk_ty(self, t);
    }

    fn visit_item(&mut self, item: &'tcx rustc_hir::Item<'tcx>) {
        if generated_code(self.sess, item.span) {
            return;
        }
        if let rustc_hir::ItemKind::Fn(..) = item.kind{
            // println!("Function name is {:#?}", item.ident.name);
            let fn_sig = self.tcx.fn_sig(self.tcx.hir().local_def_id(item.hir_id()).to_def_id()).skip_binder();
            let fn_sig_str = format!("{}", fn_sig);
            if fn_sig_str.contains("secret_integers::U8") {
                let def_id = self.tcx.hir().local_def_id(item.hir_id()).to_def_id();
                let span_info = item.span;
                self.fn_defs.insert(def_id, FnDef{ident: item.ident, span:span_info});
            }
        }

        intravisit::walk_item(self, item);
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