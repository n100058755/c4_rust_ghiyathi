/// Parses a token list representing a single main function.
/// Panics if syntax is incorrect.

use crate::codegen::{ASTNode, Expr};
use crate::lexer::Token;
use std::iter::Peekable;
use std::slice::Iter;

pub fn parse(tokens: &[Token]) -> ASTNode {
    let mut iter = tokens.iter().peekable();

    match (iter.next(), iter.next(), iter.next(), iter.next(), iter.next()) {
        (Some(Token::Int), Some(Token::Identifier(_)), Some(Token::LParen), Some(Token::RParen), Some(Token::LBrace)) => {}
        _ => panic!("Syntax error in function declaration"),
    }

    match iter.next() {
        Some(Token::Return) => {
            let expr = parse_expr(&mut iter);
            match iter.next() {
                Some(Token::Semicolon) => {}
                _ => panic!("Missing semicolon after return"),
            }
            match iter.next() {
                Some(Token::RBrace) => {}
                _ => panic!("Missing closing brace"),
            }
            ASTNode::Return(expr)
        }
        _ => panic!("Expected return"),
    }
}

fn parse_expr(iter: &mut Peekable<Iter<Token>>) -> Box<Expr> {
    parse_add_sub(iter)
}

fn parse_add_sub(iter: &mut Peekable<Iter<Token>>) -> Box<Expr> {
    let mut left = parse_mul_div(iter);

    while let Some(token) = iter.peek() {
        match token {
            Token::Plus => {
                iter.next(); // consume '+'
                let right = parse_mul_div(iter);
                left = Box::new(Expr::Add(left, right));
            }
            Token::Minus => {
                iter.next(); // consume '-'
                let right = parse_mul_div(iter);
                left = Box::new(Expr::Sub(left, right));
            }
            _ => break,
        }
    }

    left
}

fn parse_mul_div(iter: &mut Peekable<Iter<Token>>) -> Box<Expr> {
    let mut left = parse_primary(iter);

    while let Some(token) = iter.peek() {
        match token {
            Token::Star => {
                iter.next(); // consume '*'
                let right = parse_primary(iter);
                left = Box::new(Expr::Mul(left, right));
            }
            _ => break,
        }
    }

    left
}

fn parse_primary(iter: &mut Peekable<Iter<Token>>) -> Box<Expr> {
    match iter.next() {
        Some(Token::Number(n)) => Box::new(Expr::Number(*n)),
        Some(Token::LParen) => {
            let expr = parse_expr(iter);
            match iter.next() {
                Some(Token::RParen) => expr,
                _ => panic!("Expected closing parenthesis"),
            }
        }
        _ => panic!("Expected number or '('"),
    }
}
