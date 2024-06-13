pub mod binding;
pub mod error_bag;
pub mod operator;
mod parser;

use self::error_bag::{ErrorBag, ErrorKind};

pub fn analyze(contents: String) -> ErrorBag {
    // parsing
    let mut bag = ErrorBag::new();
    let root = match parser::parse_contents(contents, &mut bag) {
        Some(root) => root,
        None => {
            bag.add(ErrorKind::FailedParsing, 0, 0);
            return bag;
        }
    };

    print!("{:#?}", &root);

    // binding
    let bound = match binding::bind_root(&root, &mut bag) {
        Some(bound_root) => bound_root,
        None => return bag,
    };

    println!("{:#?}", bound);
    bag
}
