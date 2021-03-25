// This is required to be able to use the rustc crates.
#![feature(rustc_private)]

// This is only required for the `rustc_*` crates. Regular dependencies can be used without it.
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;

use rustc_driver::{catch_with_exit_code, Callbacks, Compilation, RunCompiler};
use rustc_hir::itemlikevisit::ItemLikeVisitor;
use rustc_hir::{ForeignItem, ImplItem, Item, ItemKind, TraitItem, TyKind};
use rustc_interface::{Config, interface::Compiler, Queries};
use rustc_middle::mir::TerminatorKind;
use rustc_middle::ty::TyCtxt;

// use rustc_middle::ty::TyKind;

// Custom Compiler Callbacks
pub(crate) struct CustomCallbacks;

impl Callbacks for CustomCallbacks {

    // first callback the compiler driver calls
    fn config(&mut self, config: &mut Config) {
        // prevent the compiler from dropping the expanded AST
        config.opts.debugging_opts.save_analysis = true;
    }

    fn after_analysis<'tcx>(
        &mut self,
        compiler: &Compiler,
        queries: &'tcx Queries<'tcx>
    ) -> Compilation {

        // println!("Hello from rustc after analysis");
        compiler.session().abort_if_errors();
        let crate_name = queries.crate_name().unwrap().peek();
        // let us worry only about this crate
        if *crate_name == "secret_integers_usage" {
            queries.global_ctxt().unwrap().peek_mut().enter( |tcx|{
                let hir = tcx.hir();
                let krate = hir.krate();
                let mut visitor = CustomVisitor {tcx};
                krate.visit_all_item_likes(&mut visitor); // can be done in one line. change later.
            });
        }


        Compilation::Continue
    }

}

struct CustomVisitor<'tcx> {
    tcx: TyCtxt<'tcx>,
}

impl<'hir, 'tcx> ItemLikeVisitor<'hir> for CustomVisitor<'tcx> {



    fn visit_item(&mut self, item: &'hir Item<'hir>) {
        // TODO: handle the type alias and stuff

        fn does_this_has_secret_type(fn_decl: &rustc_hir::FnDecl) -> bool {
            let mut result = false;

            return result;
        }

        // entry point like a function definition. even the main is one of them.
        // So, we can start to sweep all the functions fist and see.
        if let ItemKind::Fn(..) = item.kind {
            println!("Function name: {:#?} and span: {:#?}", item.ident, item.span);
            let def_id = self.tcx.hir().local_def_id(item.hir_id()).to_def_id();
            println!("The def id is {:#?}", def_id);
            if let Some(fn_decl) = self.tcx.hir().fn_decl_by_hir_id(item.hir_id()) {
                if fn_decl.inputs.len() > 0 {
                    if let TyKind::Rptr(_lifetime, mut_ty) = &fn_decl.inputs[0].kind {
                        println!("{:#?}", mut_ty.ty);
                    }
                }
            }
            // let mir = self.tcx.optimized_mir(def_id);
            // for bb_data in mir.basic_blocks() {
            //     match &bb_data.terminator.as_ref().unwrap().kind{
            //         TerminatorKind::Call { func, .. } => {
            //             let ty = func.ty(mir, self.tcx);
            //             if let TyKind::FnDef(def_id, _) = ty.kind() {
            //                 println!("{:?}", def_id);
            //             }
            //         }

            //         _ => (),
            //     }

            // }
        }
    }
    fn visit_trait_item(&mut self, _trait_item: &'hir TraitItem<'hir>) {}
    fn visit_impl_item(&mut self, _impl_item: &'hir ImplItem<'hir>) {}
    fn visit_foreign_item(&mut self, _foreign_item: &'hir ForeignItem<'hir>) {}
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