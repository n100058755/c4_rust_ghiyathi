#![allow(dead_code)] //suppress warnings for unused opcodes

///this module will implement a simple stack-based virtual machine for executing instructions
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    IMM(i64),
    PSH,
    ADD,
    SUB,
    MUL,
    DIV,
    MOD,
    JMP(usize),
    BZ(usize),
    BNZ(usize),
    JSR(usize),
    ENT(usize),
    ADJ(usize),
    LEV,
    LEA(usize),
    LI,
    LC,
    SI,
    SC,
    EXIT,
    MALC,
    FREE,
    MSET,
    MCMP,
    OPEN,
    READ,
    CLOS,
    EQ, // for ==
    LT, // for <
    GT, // for >
    PrintfStr(String), // for printf string
}

///simple stack-based virtual machine struct
pub struct VM {
    pub stack: Vec<i64>,
    pub pc: usize,
    pub bp: usize,
    pub program: Vec<Instruction>,
    pub running: bool,
    pub trace: bool,  
}

///execute the instructions in the program
impl VM {
    //create a new VM instance with the given program
    pub fn new(program: Vec<Instruction>) -> Self {
        VM {
            stack: Vec::new(),
            pc: 0,
            bp: 0,
            program,
            running: true,
            trace: false,
        }
    }

    pub fn enable_trace(&mut self) {
        self.trace = true;
    }

    //run the VM, executing instructions until the program counter exceeds the program length
    pub fn run(&mut self) {
        while self.running {
            if self.trace {
                eprintln!("TRACE pc={} instr={:?} stack={:?}", self.pc, self.program[self.pc], self.stack);
            }
            if self.pc >= self.program.len() {
                panic!("Program counter out of bounds");
            }

            match &self.program[self.pc] {
                Instruction::IMM(val) => {
                    self.stack.push(*val);
                }
                Instruction::PSH => {
                    if let Some(&top) = self.stack.last() {
                        self.stack.push(top);
                    } else {
                        panic!("PSH failed: stack is empty");
                    }
                }
                Instruction::ADD => {
                    let b = self.stack.pop().expect("ADD: missing operand B");
                    let a = self.stack.pop().expect("ADD: missing operand A");
                    self.stack.push(a + b);
                }
                Instruction::SUB => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(a - b);
                }
                Instruction::MUL => {
                    let b = self.stack.pop().expect("MUL: missing operand B");
                    let a = self.stack.pop().expect("MUL: missing operand A");
                    self.stack.push(a * b);
                }
                Instruction::DIV => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(a / b);
                }
                Instruction::MOD => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(a % b);
                }
                Instruction::JMP(target) => {
                    self.pc = *target;
                    continue;
                }
                Instruction::BZ(target) => {
                    let cond = self.stack.pop().unwrap();
                    if cond == 0 {
                        self.pc = *target;
                        continue;
                    }
                }
                Instruction::BNZ(target) => {
                    let cond = self.stack.pop().unwrap();
                    if cond != 0 {
                        self.pc = *target;
                        continue;
                    }
                }
                Instruction::JSR(target) => {
                    self.stack.push((self.pc + 1) as i64);
                    self.pc = *target;
                    continue;
                }
                Instruction::ENT(size) => {
                    self.stack.push(self.bp as i64);
                    self.bp = self.stack.len();
                    self.stack.resize(self.stack.len() + size, 0);
                }
                Instruction::ADJ(n) => {
                    for _ in 0..*n {
                        self.stack.pop();
                    }
                }
                Instruction::LEV => {
                    let old_bp = self.stack[self.bp - 1];
                    self.stack.truncate(self.bp - 1);
                    self.bp = old_bp as usize;
                    self.pc = self.stack.pop().unwrap() as usize;
                    continue;
                }
                Instruction::LEA(offset) => {
                    let addr = self.bp + offset;
                    self.stack.push(addr as i64);
                }
                Instruction::LI => {
                    let addr = self.stack.pop().unwrap() as usize;
                    let val = self.stack[addr];
                    self.stack.push(val);
                }
                Instruction::LC => {
                    let addr = self.stack.pop().unwrap() as usize;
                    let val = self.stack[addr] & 0xFF;
                    self.stack.push(val);
                }
                Instruction::SI => {
                    let val = self.stack.pop().unwrap();
                    let addr = self.stack.pop().unwrap() as usize;
                    self.stack[addr] = val;
                }
                Instruction::SC => {
                    let val = self.stack.pop().unwrap() & 0xFF;
                    let addr = self.stack.pop().unwrap() as usize;
                    self.stack[addr] = val;
                }
                Instruction::EXIT => {
                    //drop the initial dummy value from ENT(0)
                    //drop dummy only if we actually reserved locals (ENT)
                    //drop the initial dummy only when the program really began with ENT(...)
                    if let Some(first) = self.program.get(0) {
                        if let Instruction::ENT(_) = *first {
                            if !self.stack.is_empty() {
                                self.stack.remove(0);
                                self.stack.remove(0);
                            }
                        }
                    }

                     //println!("Final stack: {:?}", self.stack);
                     if let Some(&result) = self.stack.last() {
                         println!("Program exited with value: {}", result);
                     } else {
                         println!("Program exited: stack is empty");
                     }
                     self.running = false;
                 }



                Instruction::PrintfStr(s) => {
                    print!("{}", s);
                }
                Instruction::MALC => {
                    //MALC takes two inputs (size, flags) pop them both
                    let _flags = self.stack.pop().expect("MALC missing flags");
                    let _size  = self.stack.pop().expect("MALC missing size");
                    //push an error/status code of 0, then the pointer
                    self.stack.push(0);
                    self.stack.push(0x1000);

                }
                Instruction::FREE => {
                    let _ = self.stack.pop();
                }
                Instruction::MSET => {
                    let _ = self.stack.pop();
                    let _ = self.stack.pop();
                    let _ = self.stack.pop();
                }
                Instruction::MCMP => {
                    let _ = self.stack.pop();
                    let _ = self.stack.pop();
                    let _ = self.stack.pop();
                    self.stack.push(0);
                }
                Instruction::OPEN => {
                    let _ = self.stack.pop();
                    let _ = self.stack.pop();
                    self.stack.push(3);
                }
                Instruction::READ => {
                    let _ = self.stack.pop();
                    let _ = self.stack.pop();
                    let _ = self.stack.pop();
                    self.stack.push(10);
                }
                Instruction::CLOS => {
                    let _ = self.stack.pop();
                    self.stack.push(0);
                }
                Instruction::EQ => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push((a == b) as i64);
                }
                Instruction::LT => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push((a < b) as i64);
                }
                Instruction::GT => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push((a > b) as i64);
                }
            }

            self.pc += 1;
        }
    }
}

pub fn generate_instructions_from_ast(_ast: bool) -> Vec<Instruction> {
    vec![
        Instruction::IMM(7),
        Instruction::IMM(8),
        Instruction::ADD,
        Instruction::EXIT,
    ]
}
