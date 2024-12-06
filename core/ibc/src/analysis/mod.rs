pub mod binding;
pub mod control_flow;
pub mod error_bag;
pub mod operator;
pub mod span;
pub mod syntax;

use self::binding::bound_node::BoundNode;
use self::error_bag::{ErrorBag, ErrorKind};

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
    let root = match syntax::parse(contents, &mut bag) {
        Some(root) => root,
        None => {
            bag.add(ErrorKind::FailedParsing, span::Span::new(0, 0, 0, 0, 0, 0));
            return AnalysisResult::new_err(bag);
        }
    };

    // print!("{:#?}", &root);

    // binding
    let bound = match binding::bind_root(&root, &mut bag) {
        Some(bound_root) => bound_root,
        None => return AnalysisResult::new_err(bag),
    };

    // println!("{:#?}", bound);

    // control flow analysis
    let graphs = control_flow::analyze(&bound, &mut bag);
    control_flow::digraph(&graphs, "controlflow.dot");

    AnalysisResult::new(bag, bound)
}
