
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
    Plus,
    Star,
    Minus,
    Divide,
    Mod,
    Equal,
    Less,
    Greater,
    If,
    Else,
    While,
    Assign,
    Comma,
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
            '+' => {
                chars.next();
                tokens.push(Token::Plus);
            }
            '*' => {
                chars.next();
                tokens.push(Token::Star);
            }

            '-' => {
                chars.next();
                tokens.push(Token::Minus);
            }

            '/' => {
                chars.next();
                tokens.push(Token::Divide);
            }

            '%' => {
                chars.next();
                tokens.push(Token::Mod);
            }

            '=' => {
                chars.next();
                if let Some('=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::Equal); // '=='
                } else {
                    tokens.push(Token::Assign); // '='
                }
            }

            '<' => {
                chars.next();
                tokens.push(Token::Less);
            }
            '>' => {
                chars.next();
                tokens.push(Token::Greater);
            }

            ',' => {
                chars.next();
                tokens.push(Token::Comma);
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
                    "if" => tokens.push(Token::If),
                    "else" => tokens.push(Token::Else),
                    "while" => tokens.push(Token::While),
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
