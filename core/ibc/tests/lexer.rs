use ibc::analysis::syntax::lexer::{lex, LexerTokenKind};

/// Lexes `src` and keeps just the token kinds, which is what most of these
/// assertions care about.
fn kinds(src: &str) -> Vec<LexerTokenKind> {
    let tokens = lex(src.to_string());
    let mut kinds: Vec<LexerTokenKind> = Vec::new();

    for token in tokens {
        kinds.push(token.kind);
    }

    kinds
}

#[test]
fn dollar_is_never_part_of_an_identifier() {
    // type-method parameters are declared under names beginning with '$',
    // so user code must never be able to produce one -- at either position
    assert_eq!(
        kinds("$param_item"),
        vec![
            LexerTokenKind::BadToken('$'),
            LexerTokenKind::IdentifierToken("param_item".to_string()),
        ]
    );

    let mid = kinds("my$var");
    assert!(
        mid.contains(&LexerTokenKind::BadToken('$')),
        "a '$' inside an identifier must surface as a bad token, got {:?}",
        mid
    );
}

#[test]
fn stray_symbols_become_bad_tokens() {
    for character in ['@', '%', '&', '?'] {
        let source = character.to_string();
        assert_eq!(
            kinds(&source),
            vec![LexerTokenKind::BadToken(character)],
            "{} should not start an identifier",
            character
        );
    }
}

#[test]
fn ordinary_identifiers_still_lex() {
    assert_eq!(
        kinds("my_var"),
        vec![LexerTokenKind::IdentifierToken("my_var".to_string())]
    );
    assert_eq!(
        kinds("_leading"),
        vec![LexerTokenKind::IdentifierToken("_leading".to_string())]
    );
    assert_eq!(kinds("42"), vec![LexerTokenKind::IntegerLiteralToken(42)]);
}

#[test]
fn comments_produce_no_tokens() {
    for source in ["# just a comment", "# one\n# two\n# three"] {
        assert!(
            kinds(source).is_empty(),
            "{:?} should lex to nothing",
            source
        );
    }
}


#[test]
fn a_trailing_comment_is_ignored() {
    let expected = vec![
        LexerTokenKind::OutputKeyword,
        LexerTokenKind::IntegerLiteralToken(1),
    ];

    // with and without content, and with no closing newline either way
    for source in ["output 1 # explain the output", "output 1 #"] {
        assert_eq!(kinds(source), expected, "{:?}", source);
    }
}

#[test]
fn hash_inside_a_string_is_not_a_comment() {
    assert_eq!(
        kinds("output \"# not a comment\""),
        vec![
            LexerTokenKind::OutputKeyword,
            LexerTokenKind::StringLiteralToken("# not a comment".to_string()),
        ]
    );
}

#[test]
fn a_comment_does_not_shift_later_spans() {
    // the IDE underlines diagnostics by line and offset, so a comment must
    // leave both intact for the tokens that follow it
    let tokens = lex("# hi\noutput".to_string());
    assert_eq!(tokens.len(), 1);

    let start = tokens[0].span.start;

    assert_eq!(start.line, 1, "'output' is on the second line");
    // "# hi\n" is 5 characters
    assert_eq!(start.char_offset, 5);
}

#[test]
fn code_around_a_comment_still_lexes() {
    // a comment must end at its newline, whether it leads the file or
    // sits between two statements
    assert_eq!(
        kinds("# a comment\noutput 1"),
        vec![
            LexerTokenKind::OutputKeyword,
            LexerTokenKind::IntegerLiteralToken(1),
        ]
    );

    assert_eq!(
        kinds("output 1\n# middle\noutput 2"),
        vec![
            LexerTokenKind::OutputKeyword,
            LexerTokenKind::IntegerLiteralToken(1),
            LexerTokenKind::OutputKeyword,
            LexerTokenKind::IntegerLiteralToken(2),
        ]
    );
}
