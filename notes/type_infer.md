# Type Inference / Deduction
I need the code to be well typed even for imported functions, as I will be splitting the code based on functions which will interact with secret_integers.

## Ideas
- With the help of rustc compiler, generate the MIR IR of the source code and figure out functions are taking or returning *secret_integers* type.
  - It might also be worth looking at MIRI - MIR Interpreter, as it works on control flow graphs and emulate the program.
- Directly modify the rustc compiler and work with it, this looks very hard and won't be able to maintain.

### Resources
- [rustc_typeck](https://doc.rust-lang.org/stable/nightly-rustc/rustc_typeck/index.html)
- [MIR specification](https://github.com/rust-lang/rfcs/blob/master/text/1211-mir.md)
- [miri - MIR Interpreter](https://github.com/rust-lang/miri)
- [rustc_driver::Callback](https://doc.rust-lang.org/stable/nightly-rustc/rustc_driver/trait.Callbacks.html)