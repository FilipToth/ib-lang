pub mod binding;
pub mod control_flow;
pub mod error_bag;
pub mod operator;
pub mod syntax;

use pest::iterators::Pair;

use self::binding::bound_node::BoundNode;
use self::error_bag::{ErrorBag, ErrorKind};
use self::syntax::parser;
use self::syntax::parser::Rule;

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

pub struct AnalysisResult {
    pub errors: ErrorBag,
    pub root: Option<BoundNode>,
}

impl AnalysisResult {
    pub fn new(errors: ErrorBag, root: BoundNode) -> AnalysisResult {
        AnalysisResult {
            errors: errors,
            root: Some(root),
        }
    }

    pub fn new_err(errors: ErrorBag) -> AnalysisResult {
        AnalysisResult {
            errors: errors,
            root: None,
        }
    }
}

pub fn analyze(contents: String) -> AnalysisResult {
    // parsing
    let mut bag = ErrorBag::new();
    let root = match parser::parse_contents(contents, &mut bag) {
        Some(root) => root,
        None => {
            bag.add(ErrorKind::FailedParsing, 0, 0);
            return AnalysisResult::new_err(bag);
        }
    };

    print!("{:#?}", &root);

    // binding
    let bound = match binding::bind_root(&root, &mut bag) {
        Some(bound_root) => bound_root,
        None => return AnalysisResult::new_err(bag),
    };

    println!("{:#?}", bound);

    // control flow analysis
    let graphs = control_flow::analyze(&bound, &mut bag);
    control_flow::digraph(&graphs, "controlflow.dot");

    AnalysisResult::new(bag, bound)
}
