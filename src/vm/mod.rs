///VM opcodes translated from the original C4 compiler
#[derive(Debug, Clone, Copy, PartialEq)]
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
    PRTF,
    MALC,
    FREE,
    MSET,
    MCMP,
    OPEN,
    READ,
    CLOS,
}

pub struct VM {
    pub stack: Vec<i64>,
    pub pc: usize,
    pub bp: usize,
    pub program: Vec<Instruction>,
    pub running: bool,
}

impl VM {
    pub fn new(program: Vec<Instruction>) -> Self {
        VM {
            stack: Vec::new(),
            pc: 0,
            bp: 0,
            program,
            running: true,
        }
    }

    pub fn run(&mut self) {

        while self.running {
            if self.pc >= self.program.len() {
                panic!("Program counter out of bounds");
            }

            match self.program[self.pc] {
                Instruction::IMM(val) => {
                    self.stack.push(val);
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
                    self.pc = target;
                    continue;
                }
                Instruction::BZ(target) => {
                    let cond = self.stack.pop().unwrap();
                    if cond == 0 {
                        self.pc = target;
                        continue;
                    }
                }
                Instruction::BNZ(target) => {
                    let cond = self.stack.pop().unwrap();
                    if cond != 0 {
                        self.pc = target;
                        continue;
                    }
                }
                Instruction::JSR(target) => {
                    self.stack.push((self.pc + 1) as i64);
                    self.pc = target;
                    continue;
                }
                Instruction::ENT(size) => {
                    self.stack.push(self.bp as i64);
                    self.bp = self.stack.len();
                    self.stack.resize(self.stack.len() + size, 0);
                }
                Instruction::ADJ(n) => {
                    for _ in 0..n {
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
                    println!("Final stack: {:?}", self.stack);
                    if let Some(&result) = self.stack.last() {
                        println!("Program exited with value: {}", result);
                    } else {
                        println!("Program exited: stack is empty");
                    }
                    self.running = false;
                }
                // Stub system calls
                Instruction::PRTF => {
                    let _arg_count = self.stack.pop().unwrap();
                    let _fmt_addr  = self.stack.pop().unwrap();
                    println!("[PRTF] Simulated printf");
                    self.stack.push(0);
                }
                Instruction::MALC => {
                    self.stack.push(0x1000); // fake address
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
                    self.stack.push(3); // fake fd
                }
                Instruction::READ => {
                    let _ = self.stack.pop();
                    let _ = self.stack.pop();
                    let _ = self.stack.pop();
                    self.stack.push(10); // fake read count
                }
                Instruction::CLOS => {
                    let _ = self.stack.pop();
                    self.stack.push(0); // fake success
                }
            }

            self.pc += 1;
        }
    }
}
