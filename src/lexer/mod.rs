/// Tokenizes a simple C-like source code string into a vector of tokens.
/// Supports: int, return, identifiers, numbers, and symbols.

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Int,
    Return,
    Identifier(String),
    Number(i64),
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semicolon,
    Unknown(char),
}

pub fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\n' | '\r' | '\t' => {
                chars.next();
            }
            '(' => {
                chars.next();
                tokens.push(Token::LParen);
            }
            ')' => {
                chars.next();
                tokens.push(Token::RParen);
            }
            '{' => {
                chars.next();
                tokens.push(Token::LBrace);
            }
            '}' => {
                chars.next();
                tokens.push(Token::RBrace);
            }
            ';' => {
                chars.next();
                tokens.push(Token::Semicolon);
            }
            '0'..='9' => {
                let mut num = 0;
                while let Some(c) = chars.peek() {
                    if c.is_digit(10) {
                        num = num * 10 + c.to_digit(10).unwrap() as i64;
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Number(num));
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident = String::new();
                while let Some(c) = chars.peek() {
                    if c.is_alphanumeric() || *c == '_' {
                        ident.push(*c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                match ident.as_str() {
                    "int" => tokens.push(Token::Int),
                    "return" => tokens.push(Token::Return),
                    _ => tokens.push(Token::Identifier(ident)),
                }
            }
            _ => {
                tokens.push(Token::Unknown(ch));
                chars.next();
            }
        }
    }

    tokens
}
