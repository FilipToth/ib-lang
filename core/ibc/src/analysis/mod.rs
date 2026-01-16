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

impl AnalysisResult {
    /// The tree to run, or `None` when the program did not pass analysis.
    ///
    /// A program with errors is never run. Analysis stops at the first error it
    /// cannot recover from, so what is left would be only the part before it --
    /// running that does less than the program says, without saying so.
    pub fn runnable(&self) -> Option<&BoundNode> {
        if !self.errors.errors.is_empty() {
            return None;
        }

        self.root.as_ref()
    }
}

/// Analyzes `contents` and draws its control flow graph as Graphviz DOT.
///
/// The graph is `None` when the program does not compile, in which case the
/// errors say why -- a half-bound program has no meaningful flow to draw.
pub fn control_flow_graph(contents: String) -> (ErrorBag, Option<String>) {
    let mut bag = ErrorBag::new();

    let Some(root) = syntax::parse(contents, &mut bag) else {
        bag.add(ErrorKind::FailedParsing, span::Span::new(0, 0, 0, 0, 0, 0));
        return (bag, None);
    };

    let Some(bound) = binding::bind_root(&root, &mut bag) else {
        return (bag, None);
    };

    let graphs = control_flow::analyze(&bound, &mut bag);
    if !bag.errors.is_empty() {
        return (bag, None);
    }

    (bag, Some(control_flow::dot(&graphs)))
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

    // control flow analysis. the graph itself is only built on request, by
    // `control_flow_graph`, since this runs on every keystroke through the
    // diagnostics route
    control_flow::analyze(&bound, &mut bag);

    AnalysisResult::new(bag, bound)
}
