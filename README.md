
# SplitSpectre
## Summary
This project aims to split the program into "untrusted" regualr code and "trusted" code which stores and performs operations on `secret` types. We assume secret integers comes from the crate `secret_integers`, with this implicity type annotation, we split the program at function granular level.

## Crates
This project contains few crates. All the experimental and usage examples will be moved to `old_stuff` folder.

### secret_integers_usage
This crate comprises a basic crypto library in `src/simple.rs` which exposes function to get secret_key, encrypt, decrypt. `src/main.rs` has a simple code which used the `src/simple.rs` library to perform basic operations.

### split_prototype
This crate has the code manually split into "untrusted" code and "trusted" code.
"untrusted" code comprises `src/main.rs` and `src/agent_simple.rs`.
"trusted" code comprises `src/agent_server.rs` and `src/simple.rs`(original lib). Currently, "untrusted" and "trusted" code communicate through Unix Domain Sockets. The communication channel can be changed according to the needs.
Two binaries `split_prototype` and `agent_server` are generated when compiled.

### myrustc
This is a compiler extension which uses the `rustc_driver::Callbacks` to perform analysis and emit compiler diagnostics which will be consumed by tools like `cargo fix` or `rustfix`.
Binary will be present in `target/debug/myrustc`.

## Build Instructions
### prerequistes
`rustup component add cargo rust-analysis rust-src rustc-dev llvm-tools-preview clippy`

### Build
1. Build `secret_integers_usage` and `split_prototype` using regular compiler:
	1. Go to splitspectre top level dir: `cd splitspectre`
	2. Compile with cargo: `cargo build`
2. Build `myrustc` from its folder:
	1. Go to myrustc crate dir: `cd splitspectre/myrustc`
	2. Compile with cargo: `cargo build`
3. Build `secret_integers_usage` using `myrustc`
	1. Find the `myrustc` binary `splitspectre/myrustc/target/debug/myrustc`
	2. update `secret_integers_usage/myrustc` file.
		- for linux it might look like `LD_LIBRARY_PATH=$LD_LIBRARY_PATH:$(rustc --print sysroot)/lib/rustlib/<x86-linux....>/lib/ <myrustc bin file>`.
	3. update `splitspectre/secret_integers_usage/.cargo/config.toml` file.
		- `rustc` field should point to `myrustc` file in the `secret_integers_usage/myrustc`.
	4. Clean Build now `cargo clean; cargo build`
		- It will emit some trace and diagnostics.
	5. Experimental fix: `cargo fix --broken-code`
		- it will modify the code.
