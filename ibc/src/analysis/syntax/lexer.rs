use std::{str::Chars, vec};

#[derive(Debug)]
pub struct SyntaxToken {
    pub kind: SyntaxTokenKind,
}

impl SyntaxToken {
    fn new(kind: SyntaxTokenKind) -> SyntaxToken {
        SyntaxToken { kind: kind }
    }
}

#[derive(Debug)]
pub enum SyntaxTokenKind {
    PlusToken,
    MinusToken,
    StarToken,
    SlashToken,
    BangToken,
    IntegerToken(i64),
    IdentifierToken(String),

    IfKeyword,
    ThenKeyword,
    EndKeyword,
    ElseKeyword,
    OutputKeyword,
    ReturnKeyword,
    FunctionKeyword,
}

impl SyntaxTokenKind {
    pub fn unary_operator_precedence(&self) -> usize {
        match self {
            SyntaxTokenKind::PlusToken => 1,
            SyntaxTokenKind::MinusToken => 1,
            SyntaxTokenKind::BangToken => 1,
            _ => 0,
        }
    }

    pub fn binary_operator_precedence(&self) -> usize {
        match self {
            SyntaxTokenKind::StarToken => 2,
            SyntaxTokenKind::SlashToken => 2,

            SyntaxTokenKind::PlusToken => 1,
            SyntaxTokenKind::MinusToken => 1,

            _ => 0,
        }
    }
}

fn lex_identifier_or_keyword(value: String) -> SyntaxToken {
    let kind = match value.as_str() {
        "if" => SyntaxTokenKind::IfKeyword,
        "then" => SyntaxTokenKind::ThenKeyword,
        "end" => SyntaxTokenKind::EndKeyword,
        "else" => SyntaxTokenKind::ElseKeyword,
        "output" => SyntaxTokenKind::OutputKeyword,
        "return" => SyntaxTokenKind::ReturnKeyword,
        "function" => SyntaxTokenKind::FunctionKeyword,
        _ => SyntaxTokenKind::IdentifierToken(value),
    };

    SyntaxToken::new(kind)
}

fn lex_rolling(iter: &mut Chars, current: char) -> SyntaxToken {
    let mut value = current.to_string();
    let is_numeric = current.is_numeric();

    loop {
        match iter.next() {
            Some(next) => {
                if !next.is_alphanumeric() {
                    break;
                }

                if !next.is_numeric() && is_numeric {
                    // rollback position to prev, or just undo
                    // .next try running 123abs, this will return
                    // token(123) token(bs), but the 2nd token
                    // should also indlude the 'a'
                    break;
                }

                value.push(next);
            }
            None => {
                break;
            }
        };
    }

    if is_numeric {
        let int_value = value.parse();
        match int_value {
            Ok(v) => {
                let kind = SyntaxTokenKind::IntegerToken(v);
                return SyntaxToken::new(kind);
            }
            Err(_) => unreachable!(),
        }
    }

    lex_identifier_or_keyword(value)
}

pub fn lex(content: String) -> Vec<SyntaxToken> {
    let mut tokens: Vec<SyntaxToken> = vec![];
    let mut chars = content.chars();

    let mut current_value: Option<String> = None;
    loop {
        let current = match chars.next() {
            Some(c) => c,
            None => break,
        };

        let token = match current {
            '+' => SyntaxToken::new(SyntaxTokenKind::PlusToken),
            '-' => SyntaxToken::new(SyntaxTokenKind::MinusToken),
            '*' => SyntaxToken::new(SyntaxTokenKind::StarToken),
            '/' => SyntaxToken::new(SyntaxTokenKind::SlashToken),
            '!' => SyntaxToken::new(SyntaxTokenKind::BangToken),
            ' ' => continue,
            _ => lex_rolling(&mut chars, current),
        };

        tokens.push(token);
    }

    if let Some(ref mut c) = current_value {
        let identifier = SyntaxToken::new(SyntaxTokenKind::IdentifierToken(c.clone()));
        tokens.push(identifier);
    }

    tokens
}
