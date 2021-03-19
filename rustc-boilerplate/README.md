# `rustc` Boilerplate

Boilerplate required to interact with the Rust compiler. When compiled, this
project produces an executable that can be used as a replacement for `rustc`.

## Requirements

Only rustup is required to run this. Rustup will automatically install the
required compiler version that is specified in the `rust-toolchain` file.

## Usage

Run `cargo run <file>` to compile a rust source file using the custom compiler
defined in this project.

For example, running `cargo run hello.rs` will compile the `hello.rs` file and
produce a `hello` executable in the current folder.
