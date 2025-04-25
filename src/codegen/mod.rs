use crate::vm::Instruction;

#[derive(Debug, PartialEq)]
pub enum ASTNode {
    Return(Box<Expr>),
    If { condition: Box<Expr>, then_branch: Box<ASTNode>, else_branch: Option<Box<ASTNode>> },
    While { condition: Box<Expr>, body: Box<ASTNode> },
    Sequence(Vec<ASTNode>),
    Declaration(String, Box<Expr>),
    Assignment(String, Box<Expr>),
    FunctionDef {
        name: String,
        params: Vec<String>,
        body: Box<ASTNode>,
    },
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Number(i64),
    Variable(String),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Mod(Box<Expr>, Box<Expr>),
    Equal(Box<Expr>, Box<Expr>),
    Less(Box<Expr>, Box<Expr>),
    Greater(Box<Expr>, Box<Expr>),
    Call(String, Vec<Expr>),
    Var(String),
}




///Generate VM instructions from parsed AST
use std::collections::HashMap;

pub fn generate_instructions(ast: &ASTNode) -> Vec<Instruction> {
    let mut instructions = Vec::new();
    let mut symbol_table = HashMap::new();
    let mut next_offset = 0;

    //Reserve local variable space
    instructions.push(Instruction::ENT(0)); // placeholder

    generate_instructions_inner(ast, &mut instructions, &mut symbol_table, &mut next_offset);

    //Patch ENT with correct stack size
    instructions[0] = Instruction::ENT(next_offset);

    instructions
}


fn generate_instructions_inner(
    ast: &ASTNode,
    instructions: &mut Vec<Instruction>,
    symbol_table: &mut HashMap<String, usize>,
    next_offset: &mut usize,
) {
    match ast {
        ASTNode::Return(expr) => {
            emit_expr(expr, instructions, symbol_table);
            instructions.push(Instruction::PSH);
            instructions.push(Instruction::EXIT);
        }

        ASTNode::If { condition, then_branch, else_branch } => {
            emit_expr(condition, instructions, symbol_table);
            let jump_false_index = instructions.len();
            instructions.push(Instruction::BZ(9999)); // placeholder

            generate_instructions_inner(then_branch, instructions, symbol_table, next_offset);

            if let Some(else_branch) = else_branch {
                let jump_over_else_index = instructions.len();
                instructions.push(Instruction::JMP(9999)); // placeholder

                let else_start = instructions.len();
                generate_instructions_inner(else_branch, instructions, symbol_table, next_offset);

                let after_else = instructions.len();
                instructions[jump_false_index] = Instruction::BZ(else_start);
                instructions[jump_over_else_index] = Instruction::JMP(after_else);
            } else {
                let after_then = instructions.len();
                instructions[jump_false_index] = Instruction::BZ(after_then);
            }
        }

        ASTNode::While { condition, body } => {
            let loop_start = instructions.len();

            emit_expr(condition, instructions, symbol_table);

            let jump_if_false_index = instructions.len();
            instructions.push(Instruction::BZ(9999)); // placeholder

            generate_instructions_inner(body, instructions, symbol_table, next_offset);

            instructions.push(Instruction::JMP(loop_start));

            let loop_end = instructions.len();
            instructions[jump_if_false_index] = Instruction::BZ(loop_end);
        }

        ASTNode::Sequence(statements) => {
            for stmt in statements {
                generate_instructions_inner(stmt, instructions, symbol_table, next_offset);
            }
        }

        ASTNode::Declaration(name, expr) => {
            let offset = *next_offset;
            *next_offset += 1;
            symbol_table.insert(name.clone(), offset);

            instructions.push(Instruction::LEA(offset));          
            emit_expr(expr, instructions, symbol_table);
            instructions.push(Instruction::SI);
        }

        ASTNode::Assignment(name, expr) => {
            if let Some(&offset) = symbol_table.get(name) {
                instructions.push(Instruction::LEA(offset));      
                emit_expr(expr, instructions, symbol_table);
                instructions.push(Instruction::SI);
            } else {
                panic!("Assignment to undeclared variable: {}", name);
            }
        }

        ASTNode::FunctionDef { name, params, body } => {
            // Just a placeholder for function entry, actual call resolution would need symbol mapping
            // ENT reserves space for local variables
            instructions.push(Instruction::ENT(params.len()));
            generate_instructions_inner(body, instructions, symbol_table, next_offset);
            // We expect every function to end with a return that calls `EXIT`
        }

    }
}



fn emit_expr(
    expr: &Expr,
    instructions: &mut Vec<Instruction>,
    symbol_table: &HashMap<String, usize>,
) {
    match expr {
        Expr::Number(n) => {
            instructions.push(Instruction::IMM(*n));
        }
        Expr::Add(lhs, rhs) => {
            emit_expr(lhs, instructions, symbol_table);
            emit_expr(rhs, instructions, symbol_table);
            instructions.push(Instruction::ADD);
        }
        Expr::Sub(lhs, rhs) => {
            emit_expr(lhs, instructions, symbol_table);
            emit_expr(rhs, instructions, symbol_table);
            instructions.push(Instruction::SUB);
        }
        Expr::Mul(lhs, rhs) => {
            emit_expr(lhs, instructions, symbol_table);
            emit_expr(rhs, instructions, symbol_table);
            instructions.push(Instruction::MUL);
        }
        Expr::Div(lhs, rhs) => {
            emit_expr(lhs, instructions, symbol_table);
            emit_expr(rhs, instructions, symbol_table);
            instructions.push(Instruction::DIV);
        }
        Expr::Mod(lhs, rhs) => {
            emit_expr(lhs, instructions, symbol_table);
            emit_expr(rhs, instructions, symbol_table);
            instructions.push(Instruction::MOD);
        }
        Expr::Equal(lhs, rhs) => {
            emit_expr(lhs, instructions, symbol_table);
            emit_expr(rhs, instructions, symbol_table);
            instructions.push(Instruction::EQ);
        }
        Expr::Less(lhs, rhs) => {
            emit_expr(lhs, instructions, symbol_table);
            emit_expr(rhs, instructions, symbol_table);
            instructions.push(Instruction::LT);
        }
        Expr::Greater(lhs, rhs) => {
            emit_expr(lhs, instructions, symbol_table);
            emit_expr(rhs, instructions, symbol_table);
            instructions.push(Instruction::GT);
        }
        Expr::Variable(name) => {
            if let Some(&offset) = symbol_table.get(name) {
                instructions.push(Instruction::LEA(offset));
                instructions.push(Instruction::LI); // load value from address
            } else {
                panic!("Use of undeclared variable: {}", name);
            }
        }
        Expr::Call(func_name, args) => {
            // Push each argument in order (left-to-right)
            for arg in args {
                emit_expr(arg, instructions, symbol_table);
            }

            // Issue JSR (jump to subroutine) with function name as label
            // for now, assuming a function label index is stored in a mapping
            let dummy_address = 1000; // placeholder address
            instructions.push(Instruction::JSR(dummy_address));
        }

        Expr::Var(name) => {
            if let Some(&offset) = symbol_table.get(name) {
                instructions.push(Instruction::LEA(offset));
                instructions.push(Instruction::LI);
            } else {
                panic!("Use of undeclared variable: {}", name);
            }
        }

    }
}
