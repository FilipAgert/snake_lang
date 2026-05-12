# Snake lang Compiler

A minimalist compiler for the snake lang programming language written in Rust.

## Requirements

You must install the Rust toolchain to build this project. Download it via [rustup.rs](https://rustup.rs/).

## Build Instructions

```bash
cargo build --release

```

## Usage

Run the compiler by providing a source file with the `-f` flag:

```bash
./target/release/snake_compiler -f <file_name>

```

## Compiler progress
The compiler is a work in progress. Current implementation of compiler pipeline.
- [x] Lexing source code into tokens
- [x] Parsing tokens into abstract syntax tree
- [ ] Semantic analysis
   - [x] Link stage
   - [x] Type checking stage
   - [ ] other stages.
- [ ] To intermediary representation (IR)
- [ ] To assembly

## Language features to implement
- While statements
- Structures
- Muteable variables




## Project Structure

* `src/`: Contains the Rust source code for the compiler.
