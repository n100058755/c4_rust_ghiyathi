# C4Rust

A minimal self-hosted C4 compiler and VM written in Rust.

---

## Overview

**C4Rust** is a teaching-quality reimplementation of the classic [C4 compiler](c4.c) in Rust. It:

- **Lexes** a tiny subset of C (variables, arithmetic, 'if'/'while', 'printf', functions)  
- **Parses** into an AST  
- **Codegens** a simple stack-machine bytecode  
- **Executes** it on a toy VM with syscalls stubbed out  
- **Self-hosts**: compiles its own 'c4.c' and demonstrates end-to-end correctness  

Built for clarity, unit-test coverage, and easy extension.

---

## Features

- **Lexer**: identifiers, numbers, strings, operators, comments  
- **Parser**: recursive-descent, expressions ('+', '-', '*', '/', '%', '==', '<', '>'), statements, 'printf'  
- **Codegen**: AST to 'Instruction' stream, including 'ENT'/'LEV' for stack frames  
- **VM**: stack-machine supporting arithmetic, control flow ('JMP', 'BZ', 'BNZ'), function calls, memory ops, stubbed syscalls  
- **CLI** via ['clap'](https://crates.io/crates/clap):  
  - '--tokens' to dump tokens  
  - '--ast' to dump AST  
  - '--trace' to step through VM execution  
- **Unit tests**: >95% coverage on lexer, parser, codegen & VM  

---

## Getting Started

### Prerequisites

- Rust toolchain (via [rustup](https://rustup.rs))  
- 'cargo' on your 'PATH'

### Build & Test

'''bash
git clone https://github.com/n100058755/c4_rust_ghiyathi.git
cd c4_rust_ghiyathi

# run all unit tests
cargo test
