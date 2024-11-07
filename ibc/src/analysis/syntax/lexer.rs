use std::{iter::Peekable, str::Chars, vec};

#[derive(Debug)]
pub struct LexerToken {
    pub kind: LexerTokenKind,
}

impl LexerToken {
    fn new(kind: LexerTokenKind) -> LexerToken {
        LexerToken { kind: kind }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum LexerTokenKind {
    PlusToken,
    MinusToken,
    StarToken,
    SlashToken,
    BangToken,
    EqualsToken,
    ArrowToken,
    EqualsEqualsToken,
    OpenParenthesisToken,
    CloseParenthesisToken,
    CommaToken,
    ColonToken,
    IntegerLiteralToken(i64),
    IdentifierToken(String),

    IfKeyword,
    ThenKeyword,
    EndKeyword,
    ElseKeyword,
    OutputKeyword,
    ReturnKeyword,
    FunctionKeyword,
    TrueKeyword,
    FalseKeyword,
}

impl LexerTokenKind {
    pub fn unary_operator_precedence(&self) -> usize {
        match self {
            LexerTokenKind::PlusToken => 4,
            LexerTokenKind::MinusToken => 4,
            LexerTokenKind::BangToken => 4,
            _ => 0,
        }
    }

    pub fn binary_operator_precedence(&self) -> usize {
        match self {
            LexerTokenKind::StarToken => 3,
            LexerTokenKind::SlashToken => 3,

            LexerTokenKind::PlusToken => 2,
            LexerTokenKind::MinusToken => 2,

            LexerTokenKind::EqualsEqualsToken => 1,

            _ => 0,
        }
    }
}

fn lex_identifier_or_keyword(value: String) -> LexerTokenKind {
    match value.to_lowercase().as_str() {
        "if" => LexerTokenKind::IfKeyword,
        "then" => LexerTokenKind::ThenKeyword,
        "end" => LexerTokenKind::EndKeyword,
        "else" => LexerTokenKind::ElseKeyword,
        "output" => LexerTokenKind::OutputKeyword,
        "return" => LexerTokenKind::ReturnKeyword,
        "function" => LexerTokenKind::FunctionKeyword,
        "true" => LexerTokenKind::TrueKeyword,
        "false" => LexerTokenKind::FalseKeyword,
        _ => LexerTokenKind::IdentifierToken(value),
    }
}

fn lex_rolling(iter: &mut Peekable<Chars>, current: char) -> LexerTokenKind {
    let mut value = current.to_string();
    let is_numeric = current.is_numeric();

    loop {
        let peek = iter.peek();
        match peek {
            Some(next) => {
                if !(next.is_alphanumeric() || *next == '_') {
                    break;
                }

                if !next.is_numeric() && is_numeric {
                    // rollback position to prev, or just undo
                    // .next try running 123abs, this will return
                    // token(123) token(bs), but the 2nd token
                    // should also indlude the 'a'
                    break;
                }

                value.push(*next);
                iter.next();
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
                return LexerTokenKind::IntegerLiteralToken(v);
            }
            Err(_) => unreachable!(),
        }
    }

    lex_identifier_or_keyword(value)
}

pub fn lex(content: String) -> Vec<LexerToken> {
    let mut tokens: Vec<LexerToken> = vec![];
    let mut chars = content.chars().peekable();

    let mut current_value: Option<String> = None;
    loop {
        let current = match chars.next() {
            Some(c) => c,
            None => break,
        };

        let kind = match current {
            '+' => LexerTokenKind::PlusToken,
            '-' => {
                let next_peek = chars.peek();
                match next_peek {
                    Some(n) => match n {
                        '>' => {
                            chars.next();
                            LexerTokenKind::ArrowToken
                        }
                        _ => LexerTokenKind::MinusToken,
                    },
                    None => LexerTokenKind::MinusToken,
                }
            }
            '*' => LexerTokenKind::StarToken,
            '/' => LexerTokenKind::SlashToken,
            '!' => LexerTokenKind::BangToken,
            '=' => {
                let next_peek = chars.peek();
                match next_peek {
                    Some(n) => match n {
                        '=' => {
                            chars.next();
                            LexerTokenKind::EqualsEqualsToken
                        }
                        _ => LexerTokenKind::EqualsToken,
                    },
                    None => LexerTokenKind::EqualsToken,
                }
            }
            '(' => LexerTokenKind::OpenParenthesisToken,
            ')' => LexerTokenKind::CloseParenthesisToken,
            ',' => LexerTokenKind::CommaToken,
            ':' => LexerTokenKind::ColonToken,
            ' ' => continue,
            '\n' => continue,
            '\r' => continue,
            _ => lex_rolling(&mut chars, current),
        };

        let token = LexerToken::new(kind);
        tokens.push(token);
    }

    if let Some(ref mut c) = current_value {
        let identifier = LexerToken::new(LexerTokenKind::IdentifierToken(c.clone()));
        tokens.push(identifier);
    }

    tokens
}
