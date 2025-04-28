use crate::vm::Instruction;
use std::collections::HashMap;

///parses a sequence of tokens into an AST
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
///expression types for the AST
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


///generate VM instructions from parsed AST
pub fn generate_instructions(ast: &ASTNode) -> Vec<Instruction> {
    let mut instructions = Vec::new();
    let mut symbol_table = HashMap::new();
    let mut next_offset = 0;
    let mut patches: Vec<(usize, String)> = Vec::new(); //creating patches inside the function

    //reserving local variable space
    instructions.push(Instruction::ENT(0));

    generate_instructions_inner(ast, &mut instructions, &mut symbol_table, &mut next_offset, &mut patches);

    //patch all JSR calls
    let mut function_addresses = HashMap::new();
    if let ASTNode::Sequence(stmts) = ast {
        for (i, stmt) in stmts.iter().enumerate() {
            if let ASTNode::FunctionDef { name, .. } = stmt {
                function_addresses.insert(name.clone(), i);
            }
        }
    }

    for (index, func_name) in patches {
        if let Some(&addr) = function_addresses.get(&func_name) {
            instructions[index] = Instruction::JSR(addr);
        } else {
            panic!("Unresolved function call: {}", func_name);
        }
    }

    //patch ENT with final variable space
    instructions[0] = Instruction::ENT(next_offset);

    instructions
}


///recursively generates instructions from the AST
fn generate_instructions_inner(
    ast: &ASTNode,
    instructions: &mut Vec<Instruction>,
    symbol_table: &mut HashMap<String, usize>,
    next_offset: &mut usize,
    patches: &mut Vec<(usize, String)>,
) {
    match ast {
        ASTNode::Return(expr) => {
            emit_expr(expr, instructions, symbol_table, patches);
            instructions.push(Instruction::PSH);
            instructions.push(Instruction::EXIT);
        }

        ASTNode::If { condition, then_branch, else_branch } => {
            emit_expr(condition, instructions, symbol_table, patches);
            let jump_false_index = instructions.len();
            instructions.push(Instruction::BZ(9999));

            generate_instructions_inner(then_branch, instructions, symbol_table, next_offset, patches);

            if let Some(else_branch) = else_branch {
                let jump_over_else_index = instructions.len();
                instructions.push(Instruction::JMP(9999));

                let else_start = instructions.len();
                generate_instructions_inner(else_branch, instructions, symbol_table, next_offset, patches);

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

            emit_expr(condition, instructions, symbol_table, patches);

            let jump_if_false_index = instructions.len();
            instructions.push(Instruction::BZ(9999));

            generate_instructions_inner(body, instructions, symbol_table, next_offset, patches);

            instructions.push(Instruction::JMP(loop_start));

            let loop_end = instructions.len();
            instructions[jump_if_false_index] = Instruction::BZ(loop_end);
        }

        ASTNode::Sequence(statements) => {
            for stmt in statements {
                generate_instructions_inner(stmt, instructions, symbol_table, next_offset, patches);
            }
        }

        ASTNode::Declaration(name, expr) => {
            let offset = *next_offset;
            *next_offset += 1;
            symbol_table.insert(name.clone(), offset);

            instructions.push(Instruction::LEA(offset));          
            emit_expr(expr, instructions, symbol_table, patches);
            instructions.push(Instruction::SI);
        }

        ASTNode::Assignment(name, expr) => {
            if let Some(&offset) = symbol_table.get(name) {
                instructions.push(Instruction::LEA(offset));      
                emit_expr(expr, instructions, symbol_table, patches);
                instructions.push(Instruction::SI);
            } else {
                panic!("Assignment to undeclared variable: {}", name);
            }
        }

        ASTNode::FunctionDef { name: _, params, body } => {
            symbol_table.clear();
            *next_offset = params.len();
            for (i, param) in params.iter().enumerate() {
                symbol_table.insert(param.clone(), i);
            }

            generate_instructions_inner(body, instructions, symbol_table, next_offset, patches);
        }



    }
}


///emits instructions for a given expression
fn emit_expr(
    expr: &Expr,
    instructions: &mut Vec<Instruction>,
    symbol_table: &HashMap<String, usize>,
    patches: &mut Vec<(usize, String)>,
)
{
    match expr {
        Expr::Number(n) => {
            instructions.push(Instruction::IMM(*n));
        }
        Expr::Add(lhs, rhs) => {
            emit_expr(lhs, instructions, symbol_table, patches);
            emit_expr(rhs, instructions, symbol_table, patches);
            instructions.push(Instruction::ADD);
        }
        Expr::Sub(lhs, rhs) => {
            emit_expr(lhs, instructions, symbol_table, patches);
            emit_expr(rhs, instructions, symbol_table, patches);
            instructions.push(Instruction::SUB);
        }
        Expr::Mul(lhs, rhs) => {
            emit_expr(lhs, instructions, symbol_table, patches);
            emit_expr(rhs, instructions, symbol_table, patches);
            instructions.push(Instruction::MUL);
        }
        Expr::Div(lhs, rhs) => {
            emit_expr(lhs, instructions, symbol_table, patches);
            emit_expr(rhs, instructions, symbol_table, patches);
            instructions.push(Instruction::DIV);
        }
        Expr::Mod(lhs, rhs) => {
            emit_expr(lhs, instructions, symbol_table, patches);
            emit_expr(rhs, instructions, symbol_table, patches);
            instructions.push(Instruction::MOD);
        }
        Expr::Equal(lhs, rhs) => {
            emit_expr(lhs, instructions, symbol_table, patches);
            emit_expr(rhs, instructions, symbol_table, patches);
            instructions.push(Instruction::EQ);
        }
        Expr::Less(lhs, rhs) => {
            emit_expr(lhs, instructions, symbol_table, patches);
            emit_expr(rhs, instructions, symbol_table, patches);
            instructions.push(Instruction::LT);
        }
        Expr::Greater(lhs, rhs) => {
            emit_expr(lhs, instructions, symbol_table, patches);
            emit_expr(rhs, instructions, symbol_table, patches);
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
            for arg in args {
                emit_expr(arg, instructions, symbol_table, patches);
            }
            let placeholder_index = instructions.len();
            instructions.push(Instruction::JSR(9999)); // temporary wrong address
            patches.push((placeholder_index, func_name.clone())); // save for later patching
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
