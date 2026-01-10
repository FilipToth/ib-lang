use std::{fmt, iter::Peekable, str::Chars, usize, vec};

use crate::analysis::span::{Location, Span};

#[derive(Debug)]
pub struct LexerToken {
    pub kind: LexerTokenKind,
    pub span: Span,
}

impl LexerToken {
    fn new(kind: LexerTokenKind, span: Span) -> LexerToken {
        LexerToken {
            kind: kind,
            span: span,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum LexerTokenKind {
    PlusToken,
    MinusToken,
    StarToken,
    SlashToken,
    BangToken,
    BangEqualsToken,
    EqualsToken,
    ArrowToken,
    GreaterThanToken,
    GreaterThanEqualsToken,
    LesserThanToken,
    LesserThanEqualsToken,
    EqualsEqualsToken,
    OpenParenthesisToken,
    CloseParenthesisToken,
    CommaToken,
    ColonToken,
    DotToken,
    IntegerLiteralToken(i64),
    IdentifierToken(String),
    /// A character that cannot begin a token. Kept as a token so the parser
    /// can report it with a span rather than the lexer silently dropping it.
    BadToken(char),
    StringLiteralToken(String),

    IfKeyword,
    ThenKeyword,
    EndKeyword,
    ElseKeyword,
    OutputKeyword,
    ReturnKeyword,
    FunctionKeyword,
    TrueKeyword,
    FalseKeyword,
    NewKeyword,
    LoopKeyword,
    FromKeyword,
    ToKeyword,
    ForKeyword,
    WhileKeyword,
    UntilKeyword,
    AndKeyword,
    OrKeyword,
    NotKeyword,
    ModKeyword,
    DivKeyword,
}

impl LexerTokenKind {
    pub fn unary_operator_precedence(&self) -> usize {
        match self {
            LexerTokenKind::PlusToken => 8,
            LexerTokenKind::MinusToken => 8,
            LexerTokenKind::BangToken => 8,
            // looser than comparison: `NOT X == 7` is `NOT (X == 7)`
            LexerTokenKind::NotKeyword => 3,
            _ => 0,
        }
    }

    pub fn binary_operator_precedence(&self) -> usize {
        match self {
            LexerTokenKind::StarToken => 7,
            LexerTokenKind::SlashToken => 7,
            LexerTokenKind::ModKeyword => 7,
            LexerTokenKind::DivKeyword => 7,

            LexerTokenKind::PlusToken => 6,
            LexerTokenKind::MinusToken => 6,

            LexerTokenKind::GreaterThanToken => 5,
            LexerTokenKind::GreaterThanEqualsToken => 5,
            LexerTokenKind::LesserThanToken => 5,
            LexerTokenKind::LesserThanEqualsToken => 5,

            LexerTokenKind::EqualsEqualsToken => 4,
            LexerTokenKind::BangEqualsToken => 4,

            LexerTokenKind::AndKeyword => 2,

            LexerTokenKind::OrKeyword => 1,

            _ => 0,
        }
    }
}

impl fmt::Display for LexerTokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
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
        "new" => LexerTokenKind::NewKeyword,
        "loop" => LexerTokenKind::LoopKeyword,
        "from" => LexerTokenKind::FromKeyword,
        "to" => LexerTokenKind::ToKeyword,
        "for" => LexerTokenKind::ForKeyword,
        "while" => LexerTokenKind::WhileKeyword,
        "until" => LexerTokenKind::UntilKeyword,
        "and" => LexerTokenKind::AndKeyword,
        "or" => LexerTokenKind::OrKeyword,
        "not" => LexerTokenKind::NotKeyword,
        "mod" => LexerTokenKind::ModKeyword,
        "div" => LexerTokenKind::DivKeyword,
        _ => LexerTokenKind::IdentifierToken(value),
    }
}

fn lex_rolling(
    iter: &mut Peekable<Chars>,
    current: char,
    column: &mut usize,
    char_offset: &mut usize,
) -> LexerTokenKind {
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

                *column += 1;
                *char_offset += 1;

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

fn lex_string(
    iter: &mut Peekable<Chars>,
    column: &mut usize,
    char_offset: &mut usize,
) -> LexerTokenKind {
    // current value is "
    let mut value = String::new();

    loop {
        let peek = iter.peek();
        match peek {
            Some(next) => {
                if *next == '\"' {
                    iter.next();
                    break;
                }

                // TODO: Handle line breaks

                *column += 1;
                *char_offset += 1;

                value.push(*next);
                iter.next();
            }
            None => {
                // TODO: report error, unclosed string
                break;
            }
        }
    }

    let kind = LexerTokenKind::StringLiteralToken(value);
    return kind;
}

pub fn lex(content: String) -> Vec<LexerToken> {
    let mut tokens: Vec<LexerToken> = vec![];
    let mut chars = content.chars().peekable();

    let mut line: usize = 0;
    let mut column: usize = 0;
    let mut char_offset: usize = 0;

    loop {
        let current = match chars.next() {
            Some(c) => c,
            None => break,
        };

        let start_loc = Location::new(line, column, char_offset);
        column += 1;
        char_offset += 1;

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
            '!' => {
                let next_peek = chars.peek();
                match next_peek {
                    Some('=') => {
                        chars.next();
                        LexerTokenKind::BangEqualsToken
                    }
                    _ => LexerTokenKind::BangToken,
                }
            }
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
            '>' => {
                let next_peek = chars.peek();
                match next_peek {
                    Some('=') => {
                        chars.next();
                        LexerTokenKind::GreaterThanEqualsToken
                    }
                    _ => LexerTokenKind::GreaterThanToken,
                }
            }
            '<' => {
                let next_peek = chars.peek();
                match next_peek {
                    Some('=') => {
                        chars.next();
                        LexerTokenKind::LesserThanEqualsToken
                    }
                    _ => LexerTokenKind::LesserThanToken,
                }
            }
            '(' => LexerTokenKind::OpenParenthesisToken,
            ')' => LexerTokenKind::CloseParenthesisToken,
            ',' => LexerTokenKind::CommaToken,
            ':' => LexerTokenKind::ColonToken,
            '.' => LexerTokenKind::DotToken,
            ' ' => continue,
            '#' => {
                // line comment: consume through to the end of the line. The
                // newline is deliberately left in the iterator so the '\n'
                // arm below still does the line/column bookkeeping.
                while let Some(next) = chars.peek() {
                    if *next == '\n' {
                        break;
                    }

                    column += 1;
                    char_offset += 1;
                    chars.next();
                }

                continue;
            }
            '\n' => {
                line += 1;
                column = 0;
                continue;
            }
            '\r' => continue,
            '"' => lex_string(&mut chars, &mut column, &mut char_offset),
            c if c.is_alphanumeric() || c == '_' => {
                lex_rolling(&mut chars, current, &mut column, &mut char_offset)
            }
            // Anything else cannot begin an identifier. '$' in particular is
            // reserved: type-method parameters are declared under names that
            // start with it, so user code must never be able to name one.
            _ => LexerTokenKind::BadToken(current),
        };

        let end_loc = Location::new(line, column, char_offset);
        let span = Span::from_loc(start_loc, end_loc);

        let token = LexerToken::new(kind, span);
        tokens.push(token);
    }

    tokens
}
