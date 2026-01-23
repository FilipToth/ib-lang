use ibc::analysis::error_bag::{ErrorBag, ErrorKind};
use ibc::analysis::syntax::lexer::lex;
use ibc::analysis::syntax::parser::{parse, MAX_NESTING_DEPTH};
use ibc::analysis::syntax::syntax_token::{SyntaxKind, SyntaxToken};

/// Parses `src` and returns the root token plus any diagnostics collected
/// along the way.
fn parse_source(src: &str) -> (Option<SyntaxToken>, ErrorBag) {
    let mut errors = ErrorBag::new();
    let tokens = lex(src.to_string());
    let root = parse(tokens, &mut errors);

    (root, errors)
}

/// Walks the tree and returns the first if-statement it finds, so a test
/// can assert on the shape of its else clause.
fn first_if_statement(token: &SyntaxToken) -> Option<&SyntaxToken> {
    if let SyntaxKind::IfStatement { .. } = token.kind {
        return Some(token);
    }

    let children: Vec<&SyntaxToken> = match &token.kind {
        SyntaxKind::Scope { subtokens } => subtokens.iter().collect(),
        SyntaxKind::FunctionDeclaration { body, .. } => vec![body.as_ref()],
        _ => Vec::new(),
    };

    for child in children {
        if let Some(found) = first_if_statement(child) {
            return Some(found);
        }
    }

    None
}

#[test]
fn if_without_else_has_no_else_body() {
    let (root, errors) = parse_source("if a then\n    output a\nend");
    let root = root.expect("should parse");

    assert_eq!(errors.errors.len(), 0);

    let if_token = first_if_statement(&root).expect("should contain an if");
    let SyntaxKind::IfStatement { else_body, .. } = &if_token.kind else {
        panic!("expected an if statement");
    };

    assert!(else_body.is_none());
}

#[test]
fn if_with_else_has_an_else_body() {
    let (root, errors) = parse_source("if a then\n    output a\nelse\n    output a\nend");
    let root = root.expect("should parse");

    assert_eq!(errors.errors.len(), 0, "else must not raise a diagnostic");

    let if_token = first_if_statement(&root).expect("should contain an if");
    let SyntaxKind::IfStatement { else_body, .. } = &if_token.kind else {
        panic!("expected an if statement");
    };

    assert!(else_body.is_some());
}

#[test]
fn else_inside_a_function_parses() {
    let src = "function f(a: Int) -> Int\n\
                   if a == 1 then\n\
                   return 1\n\
               else\n\
                   return 2\n\
               end\n\
               end";

    let (root, errors) = parse_source(src);
    let root = root.expect("should parse");

    assert_eq!(errors.errors.len(), 0);
    assert!(first_if_statement(&root).is_some());
}

#[test]
fn else_clause_may_be_empty() {
    let (root, errors) = parse_source("if a then\n    output a\nelse\nend");
    let root = root.expect("should parse");

    assert_eq!(errors.errors.len(), 0);

    let if_token = first_if_statement(&root).expect("should contain an if");
    let SyntaxKind::IfStatement { else_body, .. } = &if_token.kind else {
        panic!("expected an if statement");
    };

    assert!(else_body.is_some(), "an empty else is still an else clause");
}

#[test]
fn nested_else_if_parses_as_an_if_inside_the_else() {
    let src = "if a then\n\
                   output a\n\
               else\n\
                   if b then\n\
                       output b\n\
                   end\n\
               end";

    let (root, errors) = parse_source(src);
    let root = root.expect("should parse");

    assert_eq!(errors.errors.len(), 0);

    let outer = first_if_statement(&root).expect("should contain an if");
    let SyntaxKind::IfStatement { else_body, .. } = &outer.kind else {
        panic!("expected an if statement");
    };

    let else_body = else_body.as_ref().expect("else clause is present");
    assert!(
        first_if_statement(else_body).is_some(),
        "the else clause should hold the nested if"
    );
}

#[test]
fn missing_end_after_else_is_reported() {
    let (_, errors) = parse_source("if a then\n    output a\nelse\n    output a");

    assert!(
        errors.errors.len() > 0,
        "an unterminated else must still raise a diagnostic"
    );
}

/// True if parsing `src` reported an error with exactly this message.
fn reports(src: &str, message: &str) -> bool {
    let (_, errors) = parse_source(src);

    errors
        .errors
        .iter()
        .any(|error| error.kind.format() == message)
}

#[test]
fn a_statement_that_cannot_start_is_reported() {
    // each of these used to end the program early without a word
    let cases = [
        // a line cannot start with an operator
        "X = 1\n= 2",
        // a call is not something that can be assigned to
        "A = new Array<Int>()\nA.len() = 7",
        // at the top level there is no construct for an `end` to close
        "output 1\nend\noutput 2",
    ];

    for src in cases {
        assert!(reports(src, "Unexpected token"), "{:?}", src);
    }
}

#[test]
fn nested_type_annotations_parse() {
    let src = "function f(GRID: Array<Stack<Int>>) -> Array<Int>\n\
                   return new Array<Int>()\n\
               end\n\
               G = new Array<Array<Int>>()";

    let (root, errors) = parse_source(src);

    assert!(root.is_some());
    assert_eq!(errors.errors.len(), 0);
}

#[test]
fn an_unclosed_generic_is_reported() {
    assert!(reports(
        "A = new Array<Int()",
        "Expected token: close angle bracket '>'"
    ));
}

#[test]
fn else_if_on_one_line_continues_the_chain() {
    let src = "if a then\n\
                   output a\n\
               else if b then\n\
                   output b\n\
               else\n\
                   output c\n\
               end";

    let (root, errors) = parse_source(src);
    let root = root.expect("should parse");

    assert_eq!(errors.errors.len(), 0, "a chain closes with a single end");

    // the chained if is the only statement of the outer else branch, the same
    // tree the nested form gives, so nothing past the parser treats it apart
    let outer = first_if_statement(&root).expect("should contain an if");
    let SyntaxKind::IfStatement { else_body, .. } = &outer.kind else {
        panic!("expected an if statement");
    };

    let else_body = else_body.as_ref().expect("the chain is the else branch");
    let SyntaxKind::Scope { subtokens } = &else_body.kind else {
        panic!("expected the else branch to be a scope");
    };

    assert_eq!(subtokens.len(), 1);

    let SyntaxKind::IfStatement { else_body, .. } = &subtokens[0].kind else {
        panic!("expected the else branch to hold the chained if");
    };

    assert!(else_body.is_some(), "the final else belongs to the last link");
}

#[test]
fn an_else_if_chain_takes_exactly_one_end() {
    // a second end has no if left to close, so it is stray at the top level
    let (_, errors) = parse_source("if a then\n    output a\nelse if b then\n    output b\nend\nend");

    assert!(
        errors.errors.len() > 0,
        "a chain closed twice must be reported"
    );
}

// --- nesting depth ---
//
// the parser is recursive descent, so nesting in the source becomes nesting on
// the stack. overflowing it aborts the process instead of panicking, so these
// check the depth cap turns that into an ordinary diagnostic. the inputs are
// far past the cap on purpose: without it they abort the test binary.

/// The number of nesting errors reported, which the cap holds at one no matter
/// how deep the input goes.
fn nesting_errors(errors: &ErrorBag) -> usize {
    errors
        .errors
        .iter()
        .filter(|e| matches!(e.kind, ErrorKind::NestingTooDeep))
        .count()
}

#[test]
fn deeply_nested_parentheses_are_reported_rather_than_fatal() {
    let src = format!("X = {}1{}", "(".repeat(10_000), ")".repeat(10_000));
    let (_, errors) = parse_source(&src);

    assert_eq!(nesting_errors(&errors), 1);

    // the abandoned parse unwinds through every open construct, and none of
    // that is worth showing next to the one error that explains it
    assert_eq!(errors.errors.len(), 1);
}

#[test]
fn deeply_nested_blocks_are_reported_rather_than_fatal() {
    let src = format!(
        "{}output 1\n{}",
        "if true then\n".repeat(10_000),
        "end\n".repeat(10_000)
    );

    let (_, errors) = parse_source(&src);

    assert_eq!(nesting_errors(&errors), 1);
    assert_eq!(errors.errors.len(), 1);
}

#[test]
fn deeply_nested_generics_are_reported_rather_than_fatal() {
    let src = format!(
        "X = new {}Int{}()",
        "Array<".repeat(10_000),
        ">".repeat(10_000)
    );
    let (_, errors) = parse_source(&src);

    assert_eq!(nesting_errors(&errors), 1);
    assert_eq!(errors.errors.len(), 1);
}

#[test]
fn a_deep_else_if_chain_is_reported_rather_than_fatal() {
    // each link of a chain is an if inside the else of the one before it, so a
    // long chain recurses as deep as a stack of nested ifs would
    let src = format!(
        "if true then\n    output 1\n{}end",
        "else if true then\n    output 1\n".repeat(10_000)
    );

    let (_, errors) = parse_source(&src);

    assert_eq!(nesting_errors(&errors), 1);
    assert_eq!(errors.errors.len(), 1);
}

#[test]
fn nesting_the_parser_accepts_binds_without_overflowing() {
    // the binder and control flow analysis walk a tree the parser built, so
    // the cap bounds them too. this hands them about the deepest tree the
    // parser will accept, on the smallest stack any of it runs on. the few
    // levels of slack are the statement and the assignment around the parens,
    // which count against the cap as well.
    let deepest = MAX_NESTING_DEPTH - 6;
    let src = format!("X = {}1{}", "(".repeat(deepest), ")".repeat(deepest));

    let worker = std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(move || {
            let result = ibc::analysis::analyze(src);

            assert_eq!(result.errors.errors.len(), 0);
            assert!(result.root.is_some());
        })
        .expect("cannot start the analysis thread");

    worker
        .join()
        .expect("analysing at the cap overflowed the stack");
}
