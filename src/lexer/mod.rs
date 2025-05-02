#![allow(dead_code)] //suppress warnings for unused opcodes

///tokens that are recognized by the lexer
#[derive(Debug, PartialEq, Clone)]
pub enum Token { ///token types
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
    Div,
    StringLiteral(String),
    Unknown(char),
}


///converts source code string into a vector of tokens, using match here
pub fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();

    while let Some(&ch) = chars.peek() { //peek() returns an Option<&char>
        //match on the character
        match ch { 
            ' ' | '\n' | '\r' | '\t' => { //skip whitespace
                chars.next();
            } 
            '(' => { //lparen   
                chars.next();
                tokens.push(Token::LParen);
            }
            ')' => { //rparen
                chars.next();
                tokens.push(Token::RParen);
            }
            '{' => { //lbrace
                chars.next();
                tokens.push(Token::LBrace);
            }
            '}' => {  //rbrace
                chars.next();
                tokens.push(Token::RBrace);
            }
            ';' => { //semicolon
                chars.next();
                tokens.push(Token::Semicolon);
            }
            '0'..='9' => { //number literal
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
            '+' => { //addition
                chars.next();
                tokens.push(Token::Plus);
            }
            '*' => { //multiplication
                chars.next();
                tokens.push(Token::Star);
            }

            '-' => { //subtraction
                chars.next();
                tokens.push(Token::Minus);
            }

            '%' => { //modulus
                chars.next();
                tokens.push(Token::Mod);
            }

            '=' => { //assignment
                chars.next();
                if let Some('=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::Equal); // '=='
                } else {
                    tokens.push(Token::Assign); // '='
                }
            }

            '<' => { //less than
                chars.next();
                tokens.push(Token::Less);
            }
            '>' => { //greater than
                chars.next();
                tokens.push(Token::Greater);
            }

            ',' => { //comma
                chars.next();
                tokens.push(Token::Comma);
            }

            //string literal
            '"' => {
                chars.next(); //consume opening quote
                let mut s = String::new();
                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c == '"' {
                        //end of literal
                        break;
                    }
                    if c == '\\' {
                        //start of an escape sequence
                        if let Some(&esc) = chars.peek() {
                            chars.next(); //consume the escaped character
                            match esc {
                                'n'  => s.push('\n'),
                                't'  => s.push('\t'),
                                'r'  => s.push('\r'),
                                '\\' => s.push('\\'),
                                '"'  => s.push('"'),
                                other => {
                                    //unknown escape
                                    s.push('\\');
                                    s.push(other);
                                }
                            }
                            continue;
                        } else {
                            //trailing backslash with no char
                            s.push('\\');
                            break;
                        }
                    }
                    //normal character
                    s.push(c);
                }
                tokens.push(Token::StringLiteral(s)); //push the string literal token
            }

            '/' => {
                // consume the '/'
                chars.next();

                // line comment "//”
                if chars.peek() == Some(&'/') {
                    chars.next(); // skip second slash
                    while let Some(&c2) = chars.peek() {
                        if c2 == '\n' { break; }
                        chars.next();
                    }
                }
                // block comment "/* ... */”
                else if chars.peek() == Some(&'*') {
                    chars.next(); // skip the '*'
                    while let Some(&c2) = chars.peek() {
                        chars.next();
                        if c2 == '*' && chars.peek() == Some(&'/') {
                            chars.next(); // skip the '/'
                            break;
                        }
                    }
                }
                // a division operator
                else {
                    tokens.push(Token::Div);
                }
            }


                        // skip preprocessor directives ("#include”, "#define”, etc.)
            '#' => {
                // consume the '#'
                chars.next();
                // skip until end of line (or EOF)
                while let Some(&c2) = chars.peek() {
                    chars.next();
                    if c2 == '\n' {
                        break;
                    }
                }
            }

            'a'..='z' | 'A'..='Z' | '_' => { //identifier
                let mut ident = String::new();
                while let Some(c) = chars.peek() { 
                    if c.is_alphanumeric() || *c == '_' { //alphanumeric or underscore
                        ident.push(*c);
                        chars.next();
                    } else { //not an identifier character
                        break;
                    }
                } 
                match ident.as_str() { //match on the identifier
                    "int" => tokens.push(Token::Int),
                    "return" => tokens.push(Token::Return),
                    "if" => tokens.push(Token::If),
                    "else" => tokens.push(Token::Else),
                    "while" => tokens.push(Token::While),
                    _ => tokens.push(Token::Identifier(ident)),
                }

            }
            _ => {
                tokens.push(Token::Unknown(ch)); //unknown character
                chars.next();
            }
        }
    }

    tokens //return the vector of tokens
}
