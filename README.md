# brainfuck-rs

A fast, optimizing Brainfuck compiler and interpreter written in Rust.

This repository provides both a **library** (`brainfuck-rs`) and a **CLI application** to compile, optimize, and run Brainfuck programs. It features dynamic memory tape resizing, safe bounds checking, and compiler optimizations (instruction folding and cell-clear folding).

---

## Features

- **Compiler Optimizations**:
  - **Instruction Folding**: Consecutive additions (`+`), subtractions (`-`), pointer right shifts (`>`), and pointer left shifts (`<`) are folded into a single instruction containing the operation count.
  - **Loop Clear Optimization**: Loops designed to clear a cell (e.g. `[-]` or `[+]`) are automatically detected and compiled into a single `Clear` instruction instead of executing a loop cycle.
- **Dynamic Tape Resizing**: The tape starts with the standard 30,000 cells but dynamically expands to the right if the pointer moves past the boundary, preventing memory constraint errors.
- **Robust Error Handling**: Emits precise `CompileError` and `RuntimeError` variants, preventing crashes and underflows.
- **Standard Input/Output**: Seamlessly integrates with Rust's generic `std::io::Read` and `std::io::Write` traits, allowing for testing and embedding in other applications.

---

## Directory Structure

Here are the main components of this repository:

- [Cargo.toml](Cargo.toml) - Rust package manifest.
- [src/lib.rs](src/lib.rs) - Core logic: types, parsing/compilation, compiler optimizations, and execution VM.
- [src/main.rs](src/main.rs) - CLI entrypoint for running files or inline code.
- [tests/integration_tests.rs](tests/integration_tests.rs) - Suite of integration tests validating arithmetic, branching, dynamic tape growth, error cases, and sample programs.
- **Examples & Games**:
  - [examples/helloworld.b](examples/helloworld.b) - Classic Hello World.
  - [examples/tictactoe.b](examples/tictactoe.b) - A full Tic-Tac-Toe game in Brainfuck.
  - [src/ghost.b](src/ghost.b) - The Ghost Evade game in Brainfuck.

---

## Getting Started

### Prerequisites

Make sure you have [Rust and Cargo](https://rustup.rs/) installed.

### Build and Run CLI

You can run a Brainfuck file by passing the path as an argument. Files ending in `.b` will be read and executed:

```bash
cargo run -- examples/helloworld.b
```

Alternatively, you can pass inline Brainfuck code directly:

```bash
cargo run -- "++++++++++[>+++++++>++++++++++>+++>+<<<<-]>++.>+.+++++++..+++.>++.<<+++++++++++++++.>.+++.------.--------.>+.>."
```

To run interactive games:
```bash
cargo run -- examples/tictactoe.b
```

---

## Library Usage

You can use `brainfuck-rs` as a library in your own Rust projects. 

### API Overview

- [compile](src/lib.rs#L60) parses raw bytes and returns optimized instructions.
- [execute](src/lib.rs#L167) takes instructions and runs them using custom reader/writer streams.

### Example Code

```rust
use std::io;
use brainfuck_rs::{compile, execute};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let program = b"+++++[-]."; // Sets cell to 5, clears it, and outputs the result (0)
    
    // Compile and optimize program
    let instructions = compile(program)?;
    
    // Execute program using standard input and output
    execute(&instructions, io::stdin(), io::stdout())?;
    
    Ok(())
}
```

---

## Developer Guide

### Running Tests

This project has robust unit tests and integration tests to verify the correctness of the parser, compiler optimizations, and tape runtime.

To run the test suite:

```bash
cargo test
```

### Compiler Optimizations in Detail

During compilation in [src/lib.rs](src/lib.rs):

1. **Folding Arithmetic**: Consecutive repetitions of `+` and `-` are collapsed into `Instruction::Add(n)` and `Instruction::Sub(n)` respectively, allowing cell operations to happen in a single step instead of `n` iterations.
2. **Folding Pointer Movements**: Consecutive `<` and `>` operations are grouped into `Instruction::MoveLeft(n)` and `Instruction::MoveRight(n)`.
3. **Clear Operations**: The loop `[-]` or `[+]` is recognized at compile-time and translated to `Instruction::Clear`, which directly sets the cell value to `0` instead of looping decrementally/incrementally.
