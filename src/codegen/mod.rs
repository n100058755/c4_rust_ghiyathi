#![allow(dead_code)] //suppress warnings for unused codes

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
    Print(String),
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
    if let ASTNode::Sequence(nodes) = ast {
        if nodes.iter().all(|n| matches!(n, ASTNode::FunctionDef { .. })) {
            return vec![
                Instruction::IMM(0),
                Instruction::EXIT,
            ];
        }
    }
    let mut instrs = Vec::new();
    let mut symbol_table = HashMap::new();
    let mut next_offset = 0;
    let mut patches: Vec<(usize, String)> = Vec::new();

    instrs.push(Instruction::ENT(0));
    generate_instructions_inner(
        ast,
        &mut instrs,
        &mut symbol_table,
        &mut next_offset,
        &mut patches,
    );
    instrs[0] = Instruction::ENT(next_offset);

    let function_addresses: HashMap<String, usize> = HashMap::new();
    for (idx, name) in patches {
        if let Some(&addr) = function_addresses.get(&name) {
            instrs[idx] = Instruction::JSR(addr);
        } else {
            panic!("Unresolved call to {}", name);
        }
    }

    instrs
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
             //duplicate the return value so EXIT can see it
             instructions.push(Instruction::PSH);
             instructions.push(Instruction::EXIT);
         }
        ASTNode::Print(s) => {
            //push the literal onto the instruction stream
            instructions.push(Instruction::PrintfStr(s.clone()));
        }

        ASTNode::If { condition, then_branch, else_branch } => {
            //emit the condition expression
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
        //emit the while loop
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
        //emit the sequence of statements
        ASTNode::Sequence(statements) => {
            for stmt in statements {
                generate_instructions_inner(stmt, instructions, symbol_table, next_offset, patches);
            }
        }
        //emit the variable declaration
        ASTNode::Declaration(name, expr) => {
            let offset = *next_offset;
            *next_offset += 1;
            symbol_table.insert(name.clone(), offset);

            instructions.push(Instruction::LEA(offset));          
            emit_expr(expr, instructions, symbol_table, patches);
            instructions.push(Instruction::SI);
        }
        //emit the assignment
        ASTNode::Assignment(name, expr) => {
            if let Some(&offset) = symbol_table.get(name) {
                instructions.push(Instruction::LEA(offset));      
                emit_expr(expr, instructions, symbol_table, patches);
                instructions.push(Instruction::SI);
            } else {
                panic!("Assignment to undeclared variable: {}", name);
            }
        }
        //emit the function definition
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


//emits instructions for a given expression
fn emit_expr(
    expr: &Expr,
    instructions: &mut Vec<Instruction>,
    symbol_table: &HashMap<String, usize>,
    patches: &mut Vec<(usize, String)>,
)
{
    //match the expression type and emit corresponding instructions
    match expr {
        Expr::Number(n) => { //push the number onto the stack 
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
        Expr::Variable(name) => { //load the variable value
            if let Some(&offset) = symbol_table.get(name) {
                instructions.push(Instruction::LEA(offset));
                instructions.push(Instruction::LI); //load value from address
            } else {
                panic!("Use of undeclared variable: {}", name);
            }
        }
        Expr::Call(func_name, args) => { 
            for arg in args {
                emit_expr(arg, instructions, symbol_table, patches);
            }
            let placeholder_index = instructions.len();
            instructions.push(Instruction::JSR(9999)); //temporary wrong address
            patches.push((placeholder_index, func_name.clone())); // save for later patching
        }

        //load the variable value
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
