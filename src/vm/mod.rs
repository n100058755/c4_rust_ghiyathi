/// All VM opcodes translated from the original C4 compiler
#[derive(Debug, Clone, Copy, PartialEq)]
/// The instruction set for the stack machine
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
    EXIT,
}
/// The VM structure representing stack machine state
pub struct VM {
    pub stack: Vec<i64>,
    pub pc: usize,
    pub program: Vec<Instruction>,
    pub running: bool,
}

impl VM {
    /// Creates a new virtual machine with the given instructions
    pub fn new(program: Vec<Instruction>) -> Self {
        VM {
            stack: Vec::new(),
            pc: 0,
            program,
            running: true,
        }
    }

    /// Executes the program instructions in the VM
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
                        panic!("PSH failed: Stack is empty");
                    }
                }
                Instruction::ADD => {
                    let b = self.stack.pop().expect("Missing operand for ADD");
                    let a = self.stack.pop().expect("Missing operand for ADD");
                    self.stack.push(a + b);
                }
                Instruction::SUB => {
                    let b = self.stack.pop().expect("Missing operand for SUB");
                    let a = self.stack.pop().expect("Missing operand for SUB");
                    self.stack.push(a - b);
                }
                Instruction::MUL => {
                    let b = self.stack.pop().expect("Missing operand for MUL");
                    let a = self.stack.pop().expect("Missing operand for MUL");
                    self.stack.push(a * b);
                }
                Instruction::DIV => {
                    let b = self.stack.pop().expect("Missing operand for DIV");
                    let a = self.stack.pop().expect("Missing operand for DIV");
                    self.stack.push(a / b);
                }
                Instruction::MOD => {
                    let b = self.stack.pop().expect("Missing operand for MOD");
                    let a = self.stack.pop().expect("Missing operand for MOD");
                    self.stack.push(a % b);
                }
                Instruction::EXIT => {
                    if let Some(val) = self.stack.last() {
                        println!("Program exited with value: {:?}", val);
                    } else {
                        println!("Program exited: stack is empty");
                    }
                    self.running = false;
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
            }

            self.pc += 1;
        }
    }
}
