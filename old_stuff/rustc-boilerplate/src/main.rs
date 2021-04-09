// This is required to be able to use the rustc crates.
#![feature(rustc_private)]

// This is only required for the `rustc_*` crates. Regular dependencies can be used without it.
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;

use syn;
use quote::quote;
use std::io::Read;
use std::fs::File;
use std::io::prelude::*;

use rustc_driver::{catch_with_exit_code, Callbacks, Compilation, RunCompiler};
use rustc_hir::itemlikevisit::ItemLikeVisitor;
use rustc_hir::{ForeignItem, ImplItem, Item, ItemKind, TraitItem};
use rustc_interface::{interface::Compiler, Queries};
use rustc_middle::mir::TerminatorKind;
use rustc_middle::ty::TyCtxt;
use rustc_middle::ty::TyKind;

use std::{env::args, process::exit};

/// Custom Compiler callbacks.
pub(crate) struct CustomCallbacks;

impl Callbacks for CustomCallbacks {
    fn after_analysis<'tcx>(
        &mut self,
        _compiler: &Compiler,
        queries: &'tcx Queries<'tcx>,
    ) -> Compilation {
        println!("Hello from rustc after analysis");

        queries.global_ctxt().unwrap().peek_mut().enter(|tcx| {
            let mut visitor = CustomVisitor { tcx };
            tcx.hir().krate().visit_all_item_likes(&mut visitor);
        });



        Compilation::Stop
    }
}

struct CustomVisitor<'tcx> {
    tcx: TyCtxt<'tcx>,
}

impl<'hir, 'tcx> ItemLikeVisitor<'hir> for CustomVisitor<'tcx> {
    fn visit_item(&mut self, item: &'hir Item<'hir>) {
        if let ItemKind::Fn(..) = item.kind {
            let def_id = self.tcx.hir().local_def_id(item.hir_id).to_def_id();
            let mir = self.tcx.optimized_mir(def_id);
            for bb_data in mir.basic_blocks() {
                match &bb_data.terminator.as_ref().unwrap().kind {
                    TerminatorKind::Call { func, .. } => {
                        let ty = func.ty(mir, self.tcx);
                        if let TyKind::FnDef(def_id, _) = ty.kind() {
                            println!("{:?}", def_id);
                        }
                    }
                    _ => (),
                }
            }
        }
    }
    fn visit_trait_item(&mut self, _trait_item: &'hir TraitItem<'hir>) {}
    fn visit_impl_item(&mut self, _impl_item: &'hir ImplItem<'hir>) {}
    fn visit_foreign_item(&mut self, _foreign_item: &'hir ForeignItem<'hir>) {}
}

/// Run the compiler with custom callbacks and return the exit status code.
pub fn run_compiler(args: Vec<String>) -> i32 {
    catch_with_exit_code(move || RunCompiler::new(&args, &mut CustomCallbacks).run())
}

/// Get the path to the sysroot of the current rustup toolchain. Return `None` if the rustup
/// environment variables are not set.
fn sysroot() -> Option<String> {
    let home = option_env!("RUSTUP_HOME")?;
    let toolchain = option_env!("RUSTUP_TOOLCHAIN")?;
    Some(format!("{}/toolchains/{}", home, toolchain))
}

fn main() {
    // Get the arguments from the command line.
    let mut args: Vec<String> = args().collect();
    // Add the sysroot path to the arguments.
    args.push("--sysroot".into());
    args.push(sysroot().expect("rustup is not installed."));
    // Run the rust compiler with the arguments.
    let exit_code = run_compiler(args);
    // Exit with the exit code returned by the compiler.

    {
        let filename = "hello.rs";
        let mut file = File::open(&filename).expect("Unable to open file");

        let mut src = String::new();
        file.read_to_string(&mut src).expect("Unable to read file");

        // parse the file.
        let syntax = syn::parse_file(&src).expect("Unable to parse file");
        let mut file = File::create("foo.txt").unwrap();
        file.write_all(format!("{:#?}", syntax).as_bytes()).unwrap();

        println!("{:#?}", quote!(#syntax));
        println!("{}", quote!(#syntax));
    }

    exit(exit_code)
}