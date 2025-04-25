mod lexer;
mod parser;
mod vm;
mod codegen;

use codegen::{ASTNode, Expr, generate_instructions};

fn main() {
    let tokens = lexer::tokenize("int main() { if (1 < 2) return 42; return 0; }");
    let ast = parser::parse(&tokens);
    let program = codegen::generate_instructions(&ast);
    let mut vm = vm::VM::new(program);
    vm.run();


}

#[cfg(test)]
mod tests {
    use crate::lexer::{tokenize, Token};
    use crate::parser::parse;
    use crate::vm::{Instruction, VM};

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
    fn test_vm_add() {
        let program = vec![
            Instruction::IMM(2),
            Instruction::IMM(3),
            Instruction::ADD,
            Instruction::EXIT,
        ];

        let mut vm = VM::new(program);
        vm.run();

        assert_eq!(vm.stack, vec![5]);
    }

    #[test]
    fn test_vm_bz_branching() {
        let program = vec![
            Instruction::IMM(0),
            Instruction::BZ(5),
            Instruction::IMM(99),
            Instruction::IMM(100),
            Instruction::ADD,
            Instruction::IMM(42),
            Instruction::EXIT,
        ];

        let mut vm = VM::new(program);
        vm.run();

        assert_eq!(vm.stack, vec![42]);
    }

    #[test]
    fn test_vm_bnz_branching() {
        let program = vec![
            Instruction::IMM(1),
            Instruction::BNZ(5),
            Instruction::IMM(99),
            Instruction::IMM(100),
            Instruction::ADD,
            Instruction::IMM(88),
            Instruction::EXIT,
        ];

        let mut vm = VM::new(program);
        vm.run();

        assert_eq!(vm.stack, vec![88]);
    }

    #[test]
    fn test_vm_function_call() {
        let program = vec![
            Instruction::JSR(4),
            Instruction::IMM(42),
            Instruction::PSH,
            Instruction::EXIT,
            Instruction::ENT(0),
            Instruction::LEV,
        ];

        let mut vm = VM::new(program);
        vm.run();

        assert_eq!(vm.stack.last(), Some(&42));
    }

    #[test]
    fn test_vm_memory_access() {
        let program = vec![
            Instruction::ENT(2),
            Instruction::LEA(0),
            Instruction::IMM(99),
            Instruction::SI,
            Instruction::LEA(0),
            Instruction::LI,
            Instruction::EXIT,
        ];

        let mut vm = VM::new(program);
        vm.run();

        assert_eq!(vm.stack.last(), Some(&99));
    }

    #[test]
    fn test_vm_syscall_stubs() {
        let program = vec![
            Instruction::IMM(100),
            Instruction::IMM(1),
            Instruction::PRTF,
            Instruction::MALC,
            Instruction::IMM(3),
            Instruction::CLOS,
            Instruction::EXIT,
        ];

        let mut vm = VM::new(program);
        vm.run();

        assert_eq!(vm.stack, vec![0, 0x1000, 0]);
    }

    #[test]
    fn test_parser_return_add() {
        use crate::codegen::{ASTNode, Expr};

        let tokens = tokenize("int main() { return 2 + 3; }");
        let ast = parse(&tokens);
        assert_eq!(
            ast,
            ASTNode::Sequence(vec![
                ASTNode::Return(Box::new(Expr::Add(
                    Box::new(Expr::Number(2)),
                    Box::new(Expr::Number(3))
                )))
            ])
        );
    }

    #[test]
    fn test_codegen_add() {
        use crate::codegen::{generate_instructions, ASTNode, Expr};
        use crate::vm::Instruction;

        let ast = ASTNode::Sequence(vec![ASTNode::Return(Box::new(Expr::Add(
            Box::new(Expr::Number(2)),
            Box::new(Expr::Number(3)),
        )))]);

        let instructions = generate_instructions(&ast);

        assert_eq!(
            instructions,
            vec![
                Instruction::ENT(0),
                Instruction::IMM(2),
                Instruction::IMM(3),
                Instruction::ADD,
                Instruction::PSH,
                Instruction::EXIT,
            ]
        );
    }



    #[test]
    fn test_parser_add_multiply() {
        use crate::codegen::{ASTNode, Expr};

        let tokens = tokenize("int main() { return 1 + 2 * 3; }");
        let ast = parse(&tokens);

        assert_eq!(
            ast,
            ASTNode::Sequence(vec![
                ASTNode::Return(Box::new(Expr::Add(
                    Box::new(Expr::Number(1)),
                    Box::new(Expr::Mul(
                        Box::new(Expr::Number(2)),
                        Box::new(Expr::Number(3)),
                    ))
                )))
            ])
        );
    }

    #[test]
    fn test_parser_with_parentheses() {
        use crate::codegen::{ASTNode, Expr};
        let tokens = tokenize("int main() { return (1 + 2) * 3; }");
        let ast = parse(&tokens);

        assert_eq!(
            ast,
            ASTNode::Sequence(vec![
                ASTNode::Return(Box::new(Expr::Mul(
                    Box::new(Expr::Add(
                        Box::new(Expr::Number(1)),
                        Box::new(Expr::Number(2))
                    )),
                    Box::new(Expr::Number(3))
                )))
            ])
        );
    }

    #[test]
    fn test_nested_parentheses_expression() {
        use crate::codegen::{ASTNode, Expr};

        let tokens = tokenize("int main() { return (1 + 2) * (4 - 1); }");
        let ast = parse(&tokens);

        assert_eq!(
            ast,
            ASTNode::Sequence(vec![
                ASTNode::Return(Box::new(Expr::Mul(
                    Box::new(Expr::Add(
                        Box::new(Expr::Number(1)),
                        Box::new(Expr::Number(2))
                    )),
                    Box::new(Expr::Sub(
                        Box::new(Expr::Number(4)),
                        Box::new(Expr::Number(1))
                    ))
                )))
            ])
        );
    }

    #[test]
    fn test_if_else_blocks() {
        use crate::codegen::{ASTNode, Expr};

        let tokens = tokenize("int main() { if (1 < 2) { return 42; } else { return 0; } }");
        let ast = parse(&tokens);

        assert_eq!(
            ast,
            ASTNode::Sequence(vec![
                ASTNode::If {
                    condition: Box::new(Expr::Less(
                        Box::new(Expr::Number(1)),
                        Box::new(Expr::Number(2))
                    )),
                    then_branch: Box::new(ASTNode::Sequence(vec![
                        ASTNode::Return(Box::new(Expr::Number(42)))
                    ])),
                    else_branch: Some(Box::new(ASTNode::Sequence(vec![
                        ASTNode::Return(Box::new(Expr::Number(0)))
                    ])))
                }
            ])
        );
    }

    #[test]
    fn test_while_loop() {
        use crate::codegen::{ASTNode, Expr};

        let tokens = tokenize("int main() { while (1 < 2) { return 5; } }");
        let ast = parse(&tokens);

        assert_eq!(
            ast,
            ASTNode::Sequence(vec![
                ASTNode::While {
                    condition: Box::new(Expr::Less(
                        Box::new(Expr::Number(1)),
                        Box::new(Expr::Number(2)),
                    )),
                    body: Box::new(ASTNode::Sequence(vec![
                        ASTNode::Return(Box::new(Expr::Number(5)))
                    ]))
                }
            ])
        );
    }

    #[test]
    fn test_tokenizer_assignment_and_equality() {
        use crate::lexer::{tokenize, Token};

        let tokens = tokenize("int x = 5; if (x == 5) { return x; }");

        let expected = vec![
            Token::Int,
            Token::Identifier("x".to_string()),
            Token::Assign,
            Token::Number(5),
            Token::Semicolon,
            Token::If,
            Token::LParen,
            Token::Identifier("x".to_string()),
            Token::Equal,
            Token::Number(5),
            Token::RParen,
            Token::LBrace,
            Token::Return,
            Token::Identifier("x".to_string()),
            Token::Semicolon,
            Token::RBrace,
        ];

        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_var_decl_and_return() {
        use crate::lexer::tokenize;
        use crate::parser::parse;
        use crate::codegen::generate_instructions;
        use crate::vm::VM;

        let tokens = tokenize("int main() { int x = 5; return x; }");
        let ast = parse(&tokens);
        let instructions = generate_instructions(&ast);
        let mut vm = VM::new(instructions);
        vm.run();

        assert_eq!(vm.stack.last(), Some(&5));
    }

    #[test]
    fn test_codegen_function_call() {
        use crate::codegen::{generate_instructions, ASTNode, Expr};
        use crate::vm::Instruction;

        let ast = ASTNode::Return(Box::new(Expr::Call(
            "add".to_string(),
            vec![Expr::Number(2), Expr::Number(3)],
        )));

        let instructions = generate_instructions(&ast);

        assert_eq!(
            instructions,
            vec![
                Instruction::ENT(0),
                Instruction::IMM(2),
                Instruction::IMM(3),
                Instruction::JSR(1000), // placeholder
                Instruction::PSH,
                Instruction::EXIT,
            ]
        );
    }



}
