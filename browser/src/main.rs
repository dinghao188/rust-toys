use browser::dom;
use browser::parser;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let mut file = File::open("files/test.htm").unwrap();
    let mut contents = String::new();
    let a = &mut contents;
    file.read_to_string(a).expect("Error while reading");

    let html = parser::Parser::new(contents).parse();
    println!("{}", html);
}
