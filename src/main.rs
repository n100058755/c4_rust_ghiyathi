mod lexer;
mod parser;
mod vm;
mod codegen;

use codegen::{ASTNode, Expr, generate_instructions};


fn main() {
    let tokens = lexer::tokenize("int main() { return (1 + 2) * 3; }");
    let ast = parser::parse(&tokens);
    let program = codegen::generate_instructions(&ast);
    println!("Instructions: {:?}", program);

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
            Instruction::IMM(2), // Push 2 onto the stack
            Instruction::IMM(3), // Push 3 onto the stack
            Instruction::ADD,   // Add top two values
            Instruction::EXIT, // Exit
        ];
    
        let mut vm = VM::new(program);
        vm.run();
    
        assert_eq!(vm.stack, vec![5]);
    }
    
    #[test]
    fn test_vm_bz_branching() {
        use crate::vm::{Instruction, VM};
    
        let program = vec![
            Instruction::IMM(0),       // Push 0 onto the stack
            Instruction::BZ(5),        // If top == 0, jump to instruction at index 5
            Instruction::IMM(99),      // Should be skipped
            Instruction::IMM(100),     // Should be skipped
            Instruction::ADD,          // Should be skipped
            Instruction::IMM(42),      // Jump target: push 42
            Instruction::EXIT,
        ];
    
        let mut vm = VM::new(program);
        vm.run();
    
        assert_eq!(vm.stack, vec![42]);
    }
    
    #[test]
    fn test_vm_bnz_branching() {
        use crate::vm::{Instruction, VM};
    
        let program = vec![
            Instruction::IMM(1),       // Push 1 onto the stack (non-zero)
            Instruction::BNZ(5),       // Since top != 0, jump to instruction 5
            Instruction::IMM(99),      // Should be skipped
            Instruction::IMM(100),     // Should be skipped
            Instruction::ADD,          // Should be skipped
            Instruction::IMM(88),      // Jump target: push 88
            Instruction::EXIT,
        ];
    
        let mut vm = VM::new(program);
        vm.run();
    
        assert_eq!(vm.stack, vec![88]);
    }
    
    #[test]
    fn test_vm_function_call() {
        use crate::vm::{Instruction, VM};
    
        let program = vec![
            Instruction::JSR(4),        // jump to function
            Instruction::IMM(42),       // value returned manually
            Instruction::PSH,           // push return value to stack
            Instruction::EXIT,          // exit main
    
            // Function at index 4:
            Instruction::ENT(0),        // function frame
            Instruction::LEV,           // return
        ];
    
        let mut vm = VM::new(program);
        vm.run();
    
        assert_eq!(vm.stack.last(), Some(&42));
    }
    
    #[test]
    fn test_vm_memory_access() {
        use crate::vm::{Instruction, VM};
    
        let program = vec![
            Instruction::ENT(2),         // allocate 2 local variables
            Instruction::LEA(0),         // get address of var[0]
            Instruction::IMM(99),        // value to store
            Instruction::SI,             // store 99 at var[0]
            Instruction::LEA(0),         // get address of var[0]
            Instruction::LI,             // load from var[0]
            Instruction::EXIT,
        ];
    
        let mut vm = VM::new(program);
        vm.run();
    
        assert_eq!(vm.stack.last(), Some(&99));
    }
    
    #[test]
    fn test_vm_syscall_stubs() {
        use crate::vm::{Instruction, VM};
    
        let program = vec![
            // Simulate args for PRTF
            Instruction::IMM(100), // fake format address
            Instruction::IMM(1),   // fake arg count
            Instruction::PRTF,     // pops 2, pushes 0
    
            Instruction::MALC,     // pushes 0x1000 (4096)
    
            Instruction::IMM(3),   // fake file descriptor
            Instruction::CLOS,     // pops 1, pushes 0
    
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
            ASTNode::Return(Box::new(Expr::Add(
                Box::new(Expr::Number(2)),
                Box::new(Expr::Number(3))
            )))
        );
    }
    
    #[test]
    fn test_codegen_add() {
        use crate::codegen::{generate_instructions, ASTNode, Expr};
        use crate::vm::Instruction;
    
        let ast = ASTNode::Return(Box::new(Expr::Add(
            Box::new(Expr::Number(2)),
            Box::new(Expr::Number(3)),
        )));
    
        let instructions = generate_instructions(&ast);
    
        assert_eq!(
            instructions,
            vec![
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
    
        let tokens = crate::lexer::tokenize("int main() { return 1 + 2 * 3; }");
        let ast = crate::parser::parse(&tokens);
    
        assert_eq!(
            ast,
            ASTNode::Return(Box::new(Expr::Add(
                Box::new(Expr::Number(1)),
                Box::new(Expr::Mul(
                    Box::new(Expr::Number(2)),
                    Box::new(Expr::Number(3)),
                ))
            )))
        );
    }
    
    #[test]
    fn test_parser_with_parentheses() {
        use crate::codegen::{ASTNode, Expr};
        let tokens = crate::lexer::tokenize("int main() { return (1 + 2) * 3; }");
        let ast = crate::parser::parse(&tokens);
    
        assert_eq!(
            ast,
            ASTNode::Return(Box::new(Expr::Mul(
                Box::new(Expr::Add(
                    Box::new(Expr::Number(1)),
                    Box::new(Expr::Number(2))
                )),
                Box::new(Expr::Number(3))
            )))
        );
    }
    
    #[test]
    fn test_nested_parentheses_expression() {
        use crate::codegen::{ASTNode, Expr};
    
        let tokens = crate::lexer::tokenize("int main() { return (1 + 2) * (4 - 1); }");
        let ast = crate::parser::parse(&tokens);
    
        assert_eq!(
            ast,
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
        );
    }
}
