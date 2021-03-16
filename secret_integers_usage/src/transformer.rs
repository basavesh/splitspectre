use syn;
use quote::quote;
use std::io::Read;
use std::fs::File;
use std::io::prelude::*;

fn main() -> std::io::Result<()> {

    let filename = "src/main.rs";
    let mut file = File::open(&filename).expect("Unable to open file");

    let mut src = String::new();
    file.read_to_string(&mut src).expect("Unable to read file");

    // parse the file.
    let syntax = syn::parse_file(&src).expect("Unable to parse file");
    let mut file = File::create("foo.txt")?;
    file.write_all(format!("{:#?}", syntax).as_bytes())?;

    println!("{:#?}", quote!(#syntax));
    println!("{}", quote!(#syntax));
    Ok(())
}