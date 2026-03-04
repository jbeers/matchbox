# bx-rust

A high-performance, native Rust implementation of the BoxLang programming language. This project features a stack-based Bytecode Virtual Machine (VM) and a multi-stage compiler, providing a standalone runtime independent of the JVM.

## Core Features

- **Bytecode VM**: Fast, stack-based execution engine with support for nested call frames.
- **OO Support**: Full support for Classes, Objects, `this` scope, and `variables` (private) scope.
- **Modern Syntax**: Support for UDFs, Closures, Arrow functions (Lambdas), and String Interpolation.
- **Deployment**: Capability to produce standalone native and WASM binaries.

## Usage Guide

The `bx-rust` binary is a versatile tool that can interpret source code, compile to portable bytecode, or bundle applications into standalone executables.

### 1. Running Source Code (Interpreter Mode)
Run a BoxLang script (`.bxs`) directly from source. The tool will parse, compile to memory, and execute it immediately.

```bash
bx-rust my_script.bxs
```

### 2. Compiling to Bytecode
Compile source code into a compact, portable binary format (`.bxb`). This is useful for distribution where you don't want to expose source code or want to skip the parsing phase in production.

```bash
bx-rust --build my_script.bxs
# Produces: my_script.bxb
```

You can execute the resulting `.bxb` file just like a source file:
```bash
bx-rust my_script.bxb
```

### 3. Producing Standalone Native Binaries
Create a single executable file that contains both the BoxLang VM engine and your compiled code. This binary has **zero dependencies** and does not require BoxLang or Rust to be installed on the target machine.

```bash
bx-rust --target native my_script.bxs
# Produces: my_script (an executable)

./my_script
```

### 4. Producing Standalone WASM Binaries
Create a `.wasm` module with your code embedded in a custom section. This is ideal for running BoxLang in the browser, edge computing, or any WASI-compliant runtime.

*Note: Requires the `wasm32-wasip1` target to be built first.*

```bash
bx-rust --target wasm my_script.bxs
# Produces: my_script.wasm
```

### 5. Running Directory-Based Applications
`bx-rust` can treat a directory as an application. It will automatically look for an entry point (e.g., `index.bxs`, `main.bxs`, or `Application.bx`).

```bash
# Run directory app
bx-rust ./my_app_folder

# Compile directory app to a single standalone binary
bx-rust --target native ./my_app_folder
```

## Language Support Matrix

| Feature | Status | Syntax Example |
| :--- | :--- | :--- |
| **Variables** | ✅ | `x = 10`, `var y = 20` |
| **Math** | ✅ | `(10 + 5) * 2 / 3` |
| **Logic** | ✅ | `if (x > 5) { ... } else { ... }` |
| **Loops** | ✅ | `for (i=1; i<=10; i++)`, `for (item in arr)` |
| **Arrays** | ✅ | `arr = [1, 2, "three"]`, `arr[1]` (1-indexed) |
| **Structs** | ✅ | `s = { key: "val" }`, `s.key` (case-insensitive) |
| **Functions** | ✅ | `function add(a,b) { return a+b }`, `(x) => x*2` |
| **Strings** | ✅ | `"Hello #name#"`, `str1 & str2` |
| **Classes** | ✅ | `class MyClass { property p; this.p = 1; }` |
| **Exceptions**| ✅ | `try { throw "err"; } catch(e) { ... }` |

## Technical Architecture

1. **Parser**: Built using [Pest](https://pest.rs/) (PEG Grammar).
2. **Compiler**: Single-pass AST walker that emits custom 1-byte and multi-byte opcodes.
3. **VM**: Stack-based machine with a fixed-size operand stack and dynamic call frame allocation.
4. **Serialization**: Uses `bincode` for efficient binary representation of the `Chunk` (instructions + constant pool).
