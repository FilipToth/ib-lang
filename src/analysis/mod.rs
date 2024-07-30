pub mod binding;
pub mod control_flow;
pub mod error_bag;
pub mod operator;
mod parser;

use pest::iterators::Pair;

use self::error_bag::{ErrorBag, ErrorKind};
use self::parser::Rule;

#[derive(Debug, Clone)]
pub struct CodeLocation {
    line: usize,
    col: usize,
}

impl CodeLocation {
    pub fn new(line: usize, col: usize) -> CodeLocation {
        CodeLocation {
            line: line,
            col: col,
        }
    }

    pub fn from_pair(rule: &Pair<Rule>) -> CodeLocation {
        let (line, col) = rule.line_col();
        CodeLocation::new(line, col)
    }
}

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

    // control flow analysis
    let graphs = control_flow::analyze(&bound, &mut bag);
    control_flow::digraph(&graphs, "controlflow.dot");

    bag
}
