# BoxLang Rust Implementation (bx-rust)

This project is a native Rust implementation of the [BoxLang](https://github.com/ortus-boxlang/BoxLang) programming language. It aims to provide a fast, JVM-independent runtime for BoxLang scripts (`.bxs`).

## Project Vision
To create a high-performance, standalone implementation of BoxLang that focuses on syntax compatibility and built-in functions (BIFs) without the overhead of the JRE.

## Architectural Decisions

### 1. Parser: Pest over ANTLR
- **Decision:** Use [Pest](https://pest.rs/) for grammar and parsing.
- **Rationale:** Native Rust performance and idiomatic integration.
- **Location:** `src/parser/boxlang.pest` and `src/parser/mod.rs`.

### 2. Execution: Bytecode Virtual Machine (VM)
- **Decision:** Migrated from a tree-walking interpreter to a stack-based Bytecode VM.
- **Rationale:** Better support for future features (Classes, Scopes), faster execution, and portability (WASM support).
- **Location:** `src/vm/mod.rs` (VM), `src/compiler/mod.rs` (AST to Bytecode compiler).

### 3. Scoping & Types
- **Decision:** Stack-based local scoping and HashMap-based global scoping.
- **Rationale:** Efficiency and specification parity.
- **Location:** `src/types/mod.rs` and `src/vm/mod.rs`.

## Development Guidelines

### Adding New Syntax
1. Update `src/parser/boxlang.pest`.
2. Add variant to `Statement` or `Expression` in `src/ast/mod.rs`.
3. Update `src/parser/mod.rs`.
4. Update `src/compiler/mod.rs` to emit new opcodes.
5. Implement opcode in `src/vm/mod.rs`.

## Future Roadmap
- [x] Implement Stack-Based Bytecode VM.
- [x] Implement `&` for string concatenation.
- [x] Implement anonymous functions, closures, and arrow syntax.
- [x] Implement `return` statement.
- [x] Add support for `Array` and `Struct` types.
- [x] Implement for-in loops for arrays and structs.
- [x] Add support for Classes and Objects.
- [x] Implement Exception Handling (try/catch, simplified finally).
- [ ] Expand the library of BIFs.
- [ ] Add a REPL mode.
- [ ] Produce native/WASM binaries.
