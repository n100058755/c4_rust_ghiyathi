use crate::codegen::ASTNode;
use crate::lexer::Token;
use crate::Expr;
use std::iter::Peekable;
use std::slice::Iter;

///parses a sequence of tokens into an AST
pub fn parse(tokens: &[Token]) -> ASTNode {
    let mut iter = tokens.iter().peekable();
    //eprintln!("DEBUG_TOKENS = {:#?}", tokens);

    //skip everything until we see exactly 'int main() {'
    loop {
        match iter.next() {
            Some(Token::Identifier(name)) if name == "main" => {
                //consume tokens until the "{"
                while let Some(tok) = iter.next() {
                    if *tok == Token::LBrace {
                        break;
                    }
                }
                break;
            }
            Some(_) => {
                // not yet "main", keep skipping
            }
            None => panic!("couldn’t find 'main' in tokens"),
        }
    }
    let mut statements = Vec::new();
    while let Some(tok) = iter.peek() {
        match tok {
            Token::Return | Token::If | Token::While
          | Token::LBrace  | Token::Int | Token::Identifier(_) =>
                statements.push(parse_stmt(&mut iter)),
            Token::RBrace => { iter.next(); break; }
            other => panic!("Unexpected token in main body: {:?}", other),
        }
    }

    ASTNode::Sequence(statements)
}


///parses a variable declaration from the token stream
fn parse_declaration(iter: &mut Peekable<Iter<Token>>) -> ASTNode {
    let name = match iter.next() { //consume 'int'
        Some(Token::Identifier(name)) => name.clone(),
        _ => panic!("Expected variable name"),
    };

    expect_token(iter, Token::Assign); //consume '='
    let expr = parse_expr(iter); //parse the expression
    expect_token(iter, Token::Semicolon); //consume ';'

    ASTNode::Declaration(name, expr) //return the declaration
}

///parses an assignment statement from the token stream
fn parse_assignment(iter: &mut Peekable<Iter<Token>>) -> ASTNode {
    let name = match iter.next() { //consume 'int'
        Some(Token::Identifier(name)) => name.clone(),
        _ => panic!("Expected variable name"),
    };

    expect_token(iter, Token::Assign);
    let expr = parse_expr(iter); //parse the expression
    expect_token(iter, Token::Semicolon);

    ASTNode::Assignment(name, expr)
}

///parses an individual statement from the token stream
fn parse_stmt(iter: &mut Peekable<Iter<Token>>) -> ASTNode {
    //handle printf("...")
    if let Some(Token::Identifier(name)) = iter.peek() {
        if name == "printf" {
            // consume 'printf'
            iter.next();
            // consume '('
            expect_token(iter, Token::LParen);
            // next token must be a string literal
            let s = if let Some(Token::StringLiteral(s)) = iter.next() {
                s.clone()
            } else { //consume the token
                panic!("Expected string literal in printf");
            };
            expect_token(iter, Token::RParen);
            expect_token(iter, Token::Semicolon);
            return ASTNode::Print(s);
        }
    }
    match iter.peek() {
        Some(Token::Return) => {
            iter.next(); //consume 'return'
            let expr = parse_expr(iter);
            expect_token(iter, Token::Semicolon);
            ASTNode::Return(expr)
        }
        Some(Token::If) => {
            iter.next(); //consume 'if'
            parse_if(iter)
        }
        Some(Token::LBrace) => {
            parse_block(iter)
        }
        Some(Token::While) => {
            iter.next(); //consume 'while'
            parse_while(iter)
        }
        Some(Token::Int) => {
            iter.next(); //consume 'int'
            parse_declaration(iter)
        }
        Some(Token::Identifier(_)) => {
            parse_assignment(iter)
        }


        _ => panic!("Expected statement"),
    }
}

///parses a while loop from the token stream
fn parse_while(iter: &mut Peekable<Iter<Token>>) -> ASTNode {
    expect_token(iter, Token::LParen);
    let condition = parse_expr(iter);
    expect_token(iter, Token::RParen);

    let body = parse_stmt(iter); //handles both single and '{}' blocks

    ASTNode::While {
        condition,
        body: Box::new(body),
    }
}

///parses a block of statements enclosed in braces
fn parse_block(iter: &mut Peekable<Iter<Token>>) -> ASTNode {
    expect_token(iter, Token::LBrace);
    let mut stmts = Vec::new();

    while let Some(token) = iter.peek() {
        match token {
            Token::RBrace => {
                iter.next();
                break;
            }
            //also allow variable declarations ('int ...') inside blocks
            Token::Return | Token::If | Token::While | Token::LBrace | Token::Int => {
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





///parses an if statement from the token stream
fn parse_if(iter: &mut Peekable<Iter<Token>>) -> ASTNode {
    expect_token(iter, Token::LParen);
    let condition = parse_expr(iter);
    expect_token(iter, Token::RParen);

    let then_branch = parse_stmt(iter);


    let else_branch = if let Some(Token::Else) = iter.peek() {
        iter.next(); //consume 'else'
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
///parses a function call from the token stream
fn expect_token(iter: &mut Peekable<Iter<Token>>, expected: Token) {
    match iter.next() {
        Some(t) if *t == expected => {}
        other => panic!("Expected {:?}, got {:?}", expected, other),
    }
}


///parses a primary expression from the token stream
fn parse_primary(iter: &mut Peekable<Iter<Token>>) -> Box<Expr> {
    match iter.next() {
        Some(Token::Number(n)) => Box::new(Expr::Number(*n)),

        Some(Token::Identifier(name)) => {
            let name = name.clone();

            if let Some(Token::LParen) = iter.peek() {
                iter.next(); //consume '('
                let mut args = Vec::new();

                while let Some(token) = iter.peek() {
                    if let Token::RParen = token {
                        break;
                    }

                    let arg = parse_expr(iter);
                    args.push(*arg);

                    if let Some(Token::Comma) = iter.peek() {
                        iter.next(); //consume ','
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

///now handle '*' '/' '%' all at the same (high) precedence
fn parse_term(iter: &mut Peekable<Iter<Token>>) -> Box<Expr> {
    let mut node = parse_primary(iter);
    loop {
        match iter.peek() {
            Some(Token::Star) => {
                iter.next();
                let rhs = parse_primary(iter);
                node = Box::new(Expr::Mul(node, rhs));
            }
            Some(Token::Div) => {
                iter.next();
                let rhs = parse_primary(iter);
                node = Box::new(Expr::Div(node, rhs));
            }
            Some(Token::Mod) => {
                iter.next();
                let rhs = parse_primary(iter);
                node = Box::new(Expr::Mod(node, rhs));
            }
            _ => break,
        }
    }
    node
}

/// then handle '+' and '-' (lower precedence)
fn parse_add(iter: &mut Peekable<Iter<Token>>) -> Box<Expr> {
    let mut node = parse_term(iter);
    loop {
        match iter.peek() {
            Some(Token::Plus) => {
                iter.next();
                let rhs = parse_term(iter);
                node = Box::new(Expr::Add(node, rhs));
            }
            Some(Token::Minus) => {
                iter.next();
                let rhs = parse_term(iter);
                node = Box::new(Expr::Sub(node, rhs));
            }
            _ => break,
        }
    }
    node
}

fn parse_expr(iter: &mut Peekable<Iter<Token>>) -> Box<Expr> {
    parse_add(iter)
}
