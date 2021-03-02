use syn;
use quote::quote;
use std::io::Read;
use std::fs::File;

fn main() {

    let filename = "src/test_file.rs";
    let mut file = File::open(&filename).expect("Unable to open file");

    let mut src = String::new();
    file.read_to_string(&mut src).expect("Unable to read file");

    let syntax = syn::parse_file(&src).expect("Unable to parse file");

    println!("{:#?}\n", syntax);

    println!("{:#?}", quote!(#syntax));
    println!("{}", quote!(#syntax));
}