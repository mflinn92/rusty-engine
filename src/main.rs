mod dom;
mod parser;

use parser::Html;
use std::fs;

fn main() {
    let document = fs::read_to_string("test.html").unwrap();

    let dom_root = Html::parse(document);
    println!("{:#?}", dom_root);
}
