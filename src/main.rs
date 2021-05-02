mod dom;
mod parser;

use parser::Parser;
use std::fs;

fn main() {
    let html = fs::read_to_string("test.html").unwrap();

    let dom_root = Parser::parse(html);
    println!("{:#?}", dom_root);
}
