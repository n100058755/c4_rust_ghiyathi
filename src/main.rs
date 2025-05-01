mod lexer;
mod parser;
mod vm;
mod codegen;

use codegen::Expr;
use std::fs;
use clap::Parser;


///a mini C4 compiler in rust
#[derive(Parser)]
#[command(name = "c4rust", about = "Compile and run C4 programs")]
struct Cli {
    ///show tokens then exit
    #[arg(long)]
    tokens: bool,

    ///show AST then exit
    #[arg(long)]
    ast: bool,

    ///trace VM execution step by step
    #[arg(long)]
    trace: bool,

    ///input C4 source file
    input: String,
}

///main function to run the compiler
///this is the entry point for the C4 Rust compiler and VM
///reads a C file, tokenizes it, parses it into an AST
///then generates VM instructions, and runs the program
fn main() {
    //parse CLI flags
    let cli = Cli::parse();

    //read the source file
    let source = fs::read_to_string(&cli.input)
        .expect("Failed to read source file");

    //tokenize
    let tokens = lexer::tokenize(&source);
    if cli.tokens {
        println!("{:#?}", tokens);
        return;
    }

    //parse to AST
    let ast = parser::parse(&tokens);
    if cli.ast {
        println!("{:#?}", ast);
        return;
    }

    //generate a vector of VM instructions from the AST
    let program = codegen::generate_instructions(&ast);

    //create the VM
    let mut vm = vm::VM::new(program);
    if cli.trace {
        vm.enable_trace();
    }

    //run the loaded program on the VM
    vm.run();
}


///tests for the compiler
#[cfg(test)]
mod tests {

    use crate::codegen::{ASTNode, Expr};
    use crate::lexer::{tokenize, Token};
    use crate::parser::parse;
    use crate::vm::{Instruction, VM};

    #[test]
    fn test_tokenizer() {
        //verify basic tokens from a simple function definition
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
        //check that ADD instruction computes stack top values correctly
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
        //check BZ skips instructions when top of stack equals zero
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
        //check BNZ skips when top of stack is non-zero
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
        //check JSR and LEV manage function call and return value
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
        //test LEA, SI, and LI for local variable storage and retrieval
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
        //validate that placeholder syscalls push dummy values
        let program = vec![
            Instruction::IMM(100),
            Instruction::IMM(1),
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
        //parse a return statement with an expression 2+3
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
        ///esure generate_instructions outputs correct sequence for 2+3
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


    ///verify parser handles operator precedence: multiplication before addition
    #[test]
    fn test_parser_add_multiply() {
        ///verify parser handles precedence: 1 + 2 * 3

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
        ///check parser respects parentheses: (1 + 2) * 3
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
        ///test nested parentheses expression evaluation
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
        ///ensure if-else constructs parse correctly
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
        ///verify while loops parse and produce correct AST
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
        ///test tokenizer for assignment and equality operators
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
        ///test variable declaration and return statement
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
        ///test function call generation
        use crate::codegen::{generate_instructions, ASTNode, Expr};
        use crate::vm::Instruction;

        let ast = ASTNode::Sequence(vec![
            ASTNode::FunctionDef {
                name: "add".to_string(),
                params: vec!["a".to_string(), "b".to_string()],
                body: Box::new(ASTNode::Return(Box::new(Expr::Add(
                    Box::new(Expr::Variable("a".to_string())),
                    Box::new(Expr::Variable("b".to_string())),
                )))),
            },
            ASTNode::Return(Box::new(Expr::Call(
                "add".to_string(),
                vec![Expr::Number(2), Expr::Number(3)],
            ))),
        ]);

        let instructions = generate_instructions(&ast);

        assert_eq!(
            instructions,
            vec![
                Instruction::ENT(2),
                Instruction::LEA(0),
                Instruction::LI,
                Instruction::LEA(1),
                Instruction::LI,
                Instruction::ADD,
                Instruction::PSH,
                Instruction::EXIT,
                Instruction::IMM(2),
                Instruction::IMM(3),
                Instruction::JSR(0),
                Instruction::PSH,
                Instruction::EXIT,
            ]
        );
    }


    #[test]
    fn test_parser_print_statement() {
        //test print statement parsing
        let src = r#"int main() { printf("hey\n"); return 0; }"#;
        let tokens = tokenize(src);
        let ast = parse(&tokens);
        assert_eq!(
            ast,
            ASTNode::Sequence(vec![
                // printf("hey\n");
                ASTNode::Print("hey\n".to_string()),
                // return 0;
                ASTNode::Return(Box::new(Expr::Number(0))),
            ])
        );
    }

    #[test]
    fn test_parser_if_without_else() {
        //test if statement without else branch
        let src = "int main() { if (1 < 2) { return 42; } return 7; }";
        let tokens = tokenize(src);
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
                    else_branch: None,
                },
                ASTNode::Return(Box::new(Expr::Number(7))),
            ])
        );
    }


}
