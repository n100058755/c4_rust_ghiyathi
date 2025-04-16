/// Parses a token list representing a single main function.
/// Panics if syntax is incorrect.

use crate::lexer::Token;

pub fn parse(tokens: &[Token]) -> bool {
    let mut iter = tokens.iter().peekable();

    // Accept: int main() { return 42; }
    match (iter.next(), iter.next(), iter.next(), iter.next(), iter.next()) {
        (Some(Token::Int), Some(Token::Identifier(_)), Some(Token::LParen), Some(Token::RParen), Some(Token::LBrace)) => {}
        _ => panic!("Syntax error in function declaration"),
    }

    match (iter.next(), iter.next(), iter.next()) {
        (Some(Token::Return), Some(Token::Number(_)), Some(Token::Semicolon)) => {}
        _ => panic!("Syntax error in return statement"),
    }

    match iter.next() {
        Some(Token::RBrace) => true,
        _ => panic!("Missing closing brace"),
    }
}
