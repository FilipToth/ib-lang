mod parser;
mod error_bag;

use self::error_bag::{ErrorBag, ErrorKind};

pub fn analyze(contents: String) -> ErrorBag {
    let mut bag = ErrorBag::new();
    let root = match parser::parse_contents(contents, &mut bag) {
        Some(root) => root,
        None => {
            bag.add(ErrorKind::FailedParsing, 0, 0);
            return bag;
        }
    };

    println!("{:#?}", root);
    bag
}