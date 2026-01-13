use ibc::analysis::error_bag::ErrorBag;
use ibc::analysis::syntax::lexer::lex;
use ibc::analysis::syntax::parser::parse;
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
        // only the first value of a multi-value output parses
        "output 1 , 2\noutput 3",
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
