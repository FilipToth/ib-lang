#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate pest_derive;

extern crate pest;

use std::fs;

mod analysis;

fn parse_file() {
    let contents = fs::read_to_string("test.ib").unwrap();
    let bag = analysis::analyze(contents);
    bag.report();
}

fn main() {
    parse_file();
}
