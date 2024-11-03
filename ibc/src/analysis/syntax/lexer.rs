use std::vec;

#[derive(Debug)]
pub struct SyntaxToken {
    kind: SyntaxTokenKind
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
    IdentifierToken(String),
}

pub fn lex(content: String) -> Vec<SyntaxToken> {
    let mut tokens: Vec<SyntaxToken> = vec![];
    let mut chars = content.chars();

    let mut current_value: Option<String> = None;
    loop {
        let current = match chars.next() {
            Some(c) => c,
            None => break
        };

        let token = match current {
            '+' => SyntaxToken::new(SyntaxTokenKind::PlusToken),
            '-' => SyntaxToken::new(SyntaxTokenKind::MinusToken),
            '*' => SyntaxToken::new(SyntaxTokenKind::StarToken),
            '/' => SyntaxToken::new(SyntaxTokenKind::SlashToken),
            ' ' => continue,
            _ => {
                match current_value {
                    None => {
                        current_value = Some(current.to_string());
                    },
                    Some(ref mut c) => c.push(current)
                }

                continue;
            }
        };

        if let Some(ref mut c) = current_value {
            let identifier = SyntaxToken::new(SyntaxTokenKind::IdentifierToken(c.clone()));
            tokens.push(identifier);

            current_value = None;
        }

        tokens.push(token);
    }

    if let Some(ref mut c) = current_value {
        let identifier = SyntaxToken::new(SyntaxTokenKind::IdentifierToken(c.clone()));
        tokens.push(identifier);
    }

    tokens
}