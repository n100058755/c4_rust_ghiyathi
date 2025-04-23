use crate::vm::Instruction;

#[derive(Debug, PartialEq)]
pub enum ASTNode {
    Return(Box<Expr>),
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Number(i64),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),

}



///Generate VM instructions from parsed AST
pub fn generate_instructions(ast: &ASTNode) -> Vec<Instruction> {
    let mut instructions = Vec::new();

    match ast {
        ASTNode::Return(expr) => {
            emit_expr(expr, &mut instructions);
            instructions.push(Instruction::PSH);
            instructions.push(Instruction::EXIT);
        }
    }

    instructions
}

fn emit_expr(expr: &Expr, instructions: &mut Vec<Instruction>) {
    match expr {
        Expr::Number(n) => {
            instructions.push(Instruction::IMM(*n)); // Just IMM
        }
        Expr::Add(lhs, rhs) => {
            emit_expr(lhs, instructions);           // No PSH here
            emit_expr(rhs, instructions);           // No PSH here
            instructions.push(Instruction::ADD);
        }
        Expr::Mul(lhs, rhs) => {
            emit_expr(lhs, instructions);
            emit_expr(rhs, instructions);
            instructions.push(Instruction::MUL);
        }
        Expr::Sub(lhs, rhs) => {
            emit_expr(lhs, instructions);
            emit_expr(rhs, instructions);
            instructions.push(Instruction::SUB);
        }


    }
}