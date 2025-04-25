use crate::codegen::ASTNode;
use crate::lexer::Token;
use crate::Expr;
use std::iter::Peekable;
use std::slice::Iter;

pub fn parse(tokens: &[Token]) -> ASTNode {
    let mut iter = tokens.iter().peekable();

    // Parse: int main() {
    match (iter.next(), iter.next(), iter.next(), iter.next(), iter.next()) {
        (Some(Token::Int), Some(Token::Identifier(_)), Some(Token::LParen), Some(Token::RParen), Some(Token::LBrace)) => {}
        _ => panic!("Syntax error in function declaration"),
    }

    let mut statements = Vec::new();

    while let Some(token) = iter.peek() {
        match token {
            Token::Return | Token::If | Token::While | Token::LBrace | Token::Int | Token::Identifier(_) => {
                statements.push(parse_stmt(&mut iter));
            }
            Token::RBrace => {
                iter.next(); // consume '}'
                break;
            }
            _ => {
                println!("DEBUG next token in block: {:?}", token);
                panic!("Unexpected token inside block: {:?}", token);
            }
        }
    }

    ASTNode::Sequence(statements)
}



fn parse_function_def(iter: &mut Peekable<Iter<Token>>) -> ASTNode {
    expect_token(iter, Token::Int);
    let name = match iter.next() {
        Some(Token::Identifier(s)) => s.clone(),
        other => panic!("Expected function name, got {:?}", other),
    };

    expect_token(iter, Token::LParen);

    let mut params = Vec::new();
    while let Some(token) = iter.peek() {
        match token {
            Token::Int => {
                iter.next(); // consume 'int'
                match iter.next() {
                    Some(Token::Identifier(name)) => {
                        params.push(name.clone());
                        if let Some(Token::Comma) = iter.peek() {
                            iter.next(); // consume ','
                        }
                    }
                    _ => panic!("Expected parameter name"),
                }
            }
            Token::RParen => {
                iter.next(); // consume ')'
                break;
            }
            _ => panic!("Unexpected token in parameter list: {:?}", token),
        }
    }

    let body = Box::new(parse_block(iter));
    ASTNode::FunctionDef { name, params, body }
}


fn parse_declaration(iter: &mut Peekable<Iter<Token>>) -> ASTNode {
    let name = match iter.next() {
        Some(Token::Identifier(name)) => name.clone(),
        _ => panic!("Expected variable name"),
    };

    expect_token(iter, Token::Assign);
    let expr = parse_expr(iter);
    expect_token(iter, Token::Semicolon);

    ASTNode::Declaration(name, expr)
}

fn parse_assignment(iter: &mut Peekable<Iter<Token>>) -> ASTNode {
    let name = match iter.next() {
        Some(Token::Identifier(name)) => name.clone(),
        _ => panic!("Expected variable name"),
    };

    expect_token(iter, Token::Assign);
    let expr = parse_expr(iter);
    expect_token(iter, Token::Semicolon);

    ASTNode::Assignment(name, expr)
}


fn parse_stmt(iter: &mut Peekable<Iter<Token>>) -> ASTNode {
    match iter.peek() {
        Some(Token::Return) => {
            iter.next(); // consume 'return'
            let expr = parse_expr(iter);
            expect_token(iter, Token::Semicolon);
            ASTNode::Return(expr)
        }
        Some(Token::If) => {
            iter.next(); // consume 'if'
            parse_if(iter)
        }
        Some(Token::LBrace) => {
            parse_block(iter)
        }
        Some(Token::While) => {
            iter.next(); // consume 'while'
            parse_while(iter)
        }
        Some(Token::Int) => {
            iter.next(); // consume 'int'
            parse_declaration(iter)
        }
        Some(Token::Identifier(_)) => {
            parse_assignment(iter)
        }


        _ => panic!("Expected statement"),
    }
}

fn parse_while(iter: &mut Peekable<Iter<Token>>) -> ASTNode {
    expect_token(iter, Token::LParen);
    let condition = parse_expr(iter);
    expect_token(iter, Token::RParen);

    let body = parse_stmt(iter); // handles both single and `{}` blocks

    ASTNode::While {
        condition,
        body: Box::new(body),
    }
}


fn parse_block(iter: &mut Peekable<Iter<Token>>) -> ASTNode {
    expect_token(iter, Token::LBrace);
    let mut stmts = Vec::new();

    while let Some(token) = iter.peek() {
        match token {
            Token::RBrace => {
                iter.next();
                break;
            }
            Token::Return | Token::If | Token::While | Token::LBrace => {
                stmts.push(parse_stmt(iter));
            }
            t => {
                println!("DEBUG next token in block: {:?}", t);
                panic!("Unexpected token inside block: {:?}", t);
            }
        }
    }


    ASTNode::Sequence(stmts)
}






// Parses: if (<expr>) return ...; else return ...;
fn parse_if(iter: &mut Peekable<Iter<Token>>) -> ASTNode {
    expect_token(iter, Token::LParen);
    let condition = parse_expr(iter);
    expect_token(iter, Token::RParen);

    let then_branch = parse_stmt(iter);


    let else_branch = if let Some(Token::Else) = iter.peek() {
        iter.next(); // consume 'else'
        Some(Box::new(parse_stmt(iter)))
    } else {
        None
    };



    ASTNode::If {
        condition,
        then_branch: Box::new(then_branch),
        else_branch,
    }
}

// Utility: ensures the next token matches expectation
fn expect_token(iter: &mut Peekable<Iter<Token>>, expected: Token) {
    match iter.next() {
        Some(t) if *t == expected => {}
        other => panic!("Expected {:?}, got {:?}", expected, other),
    }
}

fn parse_expr(iter: &mut Peekable<Iter<Token>>) -> Box<Expr> {
    parse_cmp(iter)
}


fn parse_cmp(iter: &mut Peekable<Iter<Token>>) -> Box<Expr> {
    let mut left = parse_add_sub(iter);

    while let Some(token) = iter.peek() {
        match token {
            Token::Equal => {
                iter.next();
                let right = parse_add_sub(iter);
                left = Box::new(Expr::Equal(left, right));
            }
            Token::Less => {
                iter.next();
                let right = parse_add_sub(iter);
                left = Box::new(Expr::Less(left, right));
            }
            Token::Greater => {
                iter.next();
                let right = parse_add_sub(iter);
                left = Box::new(Expr::Greater(left, right));
            }
            _ => break,
        }
    }

    left
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
                iter.next();
                let right = parse_primary(iter);
                left = Box::new(Expr::Mul(left, right));
            }
            Token::Divide => {
                iter.next();
                let right = parse_primary(iter);
                left = Box::new(Expr::Div(left, right));
            }
            Token::Mod => {
                iter.next();
                let right = parse_primary(iter);
                left = Box::new(Expr::Mod(left, right));
            }
            _ => break,
        }
    }

    left
}

fn parse_primary(iter: &mut Peekable<Iter<Token>>) -> Box<Expr> {
    match iter.next() {
        Some(Token::Number(n)) => Box::new(Expr::Number(*n)),

        Some(Token::Identifier(name)) => {
            let name = name.clone();

            if let Some(Token::LParen) = iter.peek() {
                iter.next(); // consume '('
                let mut args = Vec::new();

                while let Some(token) = iter.peek() {
                    if let Token::RParen = token {
                        break;
                    }

                    let arg = parse_expr(iter);
                    args.push(*arg);

                    if let Some(Token::Comma) = iter.peek() {
                        iter.next(); // consume ','
                    } else {
                        break;
                    }
                }

                expect_token(iter, Token::RParen);
                Box::new(Expr::Call(name, args))
            } else {
                Box::new(Expr::Var(name))
            }
        }

        Some(Token::LParen) => {
            let expr = parse_expr(iter);
            match iter.next() {
                Some(Token::RParen) => expr,
                _ => panic!("Expected closing parenthesis"),
            }
        }

        other => panic!("Expected number, variable, or '(', got {:?}", other),
    }
}

