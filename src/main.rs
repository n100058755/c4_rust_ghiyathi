mod lexer;
mod parser;

fn main() {
    let source = "int main() { return 42; }";
    let tokens = lexer::tokenize(source);
    for token in &tokens {
        println!("{:?}", token);
    }

    let _ast = parser::parse(&tokens);
    println!("Parsing succeeded!");
}

#[cfg(test)]
mod tests {
    use crate::lexer::{tokenize, Token};
    use crate::parser::parse;

    #[test]
    fn test_tokenizer() {
        let src = "int main() { return 42; }";
        let tokens = tokenize(src);

        assert_eq!(tokens[0], Token::Int);
        assert_eq!(tokens[1], Token::Identifier("main".to_string()));
        assert_eq!(tokens[2], Token::LParen);
        assert_eq!(tokens[3], Token::RParen);
        assert_eq!(tokens[4], Token::LBrace);
        assert_eq!(tokens[5], Token::Return);
        assert_eq!(tokens[6], Token::Number(42));
        assert_eq!(tokens[7], Token::Semicolon);
        assert_eq!(tokens[8], Token::RBrace);
    }

    #[test]
    fn test_parser() {
        let tokens = tokenize("int main() { return 42; }");
        assert_eq!(parse(&tokens), true);
    }
}
