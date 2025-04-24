use crate::vm::Instruction;

#[derive(Debug, PartialEq)]
pub enum ASTNode {
    Return(Box<Expr>),
    If {
        condition: Box<Expr>,
        then_branch: Box<ASTNode>,
        else_branch: Option<Box<ASTNode>>,
    },
    Sequence(Vec<ASTNode>), // multiple statements like return; return;
}


#[derive(Debug, PartialEq)]
pub enum Expr {
    Number(i64),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Mod(Box<Expr>, Box<Expr>),
    Equal(Box<Expr>, Box<Expr>),
    Less(Box<Expr>, Box<Expr>),
    Greater(Box<Expr>, Box<Expr>),
}



///Generate VM instructions from parsed AST
pub fn generate_instructions(ast: &ASTNode) -> Vec<Instruction> {
    let mut instructions = Vec::new();
    generate_instructions_inner(ast, &mut instructions);
    instructions
}

fn generate_instructions_inner(ast: &ASTNode, instructions: &mut Vec<Instruction>) {
    match ast {
        ASTNode::Return(expr) => {
            emit_expr(expr, instructions);
            instructions.push(Instruction::PSH);
            instructions.push(Instruction::EXIT);
        }

        ASTNode::If { condition, then_branch, else_branch } => {
            emit_expr(condition, instructions);
            let jump_false_index = instructions.len();
            instructions.push(Instruction::BZ(9999)); // placeholder

            generate_instructions_inner(then_branch, instructions);

            if let Some(else_branch) = else_branch {
                let jump_over_else_index = instructions.len();
                instructions.push(Instruction::JMP(9999)); // placeholder

                let else_start = instructions.len();
                generate_instructions_inner(else_branch, instructions);

                let after_else = instructions.len();
                instructions[jump_false_index] = Instruction::BZ(else_start);
                instructions[jump_over_else_index] = Instruction::JMP(after_else);
            } else {
                let after_then = instructions.len();
                instructions[jump_false_index] = Instruction::BZ(after_then);
            }
        }

        ASTNode::Sequence(statements) => {
            for stmt in statements {
                generate_instructions_inner(stmt, instructions);
            }
        }
    }
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
        Expr::Div(lhs, rhs) => {
            emit_expr(lhs, instructions);
            emit_expr(rhs, instructions);
            instructions.push(Instruction::DIV);
        }
        Expr::Mod(lhs, rhs) => {
            emit_expr(lhs, instructions);
            emit_expr(rhs, instructions);
            instructions.push(Instruction::MOD);
        }
        Expr::Equal(lhs, rhs) => {
            emit_expr(lhs, instructions);
            emit_expr(rhs, instructions);
            instructions.push(Instruction::EQ);
        }
        Expr::Less(lhs, rhs) => {
            emit_expr(lhs, instructions);
            emit_expr(rhs, instructions);
            instructions.push(Instruction::LT);
        }
        Expr::Greater(lhs, rhs) => {
            emit_expr(lhs, instructions);
            emit_expr(rhs, instructions);
            instructions.push(Instruction::GT);
        }

    }
}
