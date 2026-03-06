# MatchBox

A high-performance, native Rust implementation of the BoxLang programming language. This project features a stack-based Bytecode Virtual Machine (VM) and a multi-stage compiler, providing a standalone runtime independent of the JVM.

## Core Features

- **Bytecode VM**: Fast, stack-based execution engine with support for nested call frames.
- **Virtual Threading (Fibers)**: High-concurrency cooperative scheduler supporting `runAsync` and non-blocking `sleep`.
- **OO Support**: Full support for Classes, Inheritance (`extends`), Interfaces (`implements`), and magic methods like `onMissingMethod`.
- **Trait-like Interfaces**: Support for default method implementations in interfaces.
- **Implicit Accessors**: Automatically generated getter and setter methods for class properties.
- **Modern Syntax**: Support for UDFs, Type Hints, Default Arguments, Closures, Arrow functions (Lambdas), and String Interpolation.
- **JS Interop (WASM)**: Direct access to JavaScript APIs and DOM manipulation when running in the browser.
- **Deployment**: Capability to produce ultra-lean (~500KB) standalone native and WASM binaries.

## Usage Guide

The `matchbox` binary is a versatile tool that can interpret source code, compile to portable bytecode, or bundle applications into standalone executables.

### 1. Running Source Code (Interpreter Mode)
Run a BoxLang script (`.bxs`) directly from source.

```bash
matchbox my_script.bxs
```

### 2. Interactive REPL
Start the BoxLang REPL by running the binary without arguments:

```bash
matchbox
```

### 3. Compiling to Bytecode
Compile source code into a compact, portable binary format (`.bxb`).

```bash
matchbox --build my_script.bxs
```

### 4. Producing Standalone Native Binaries
Create a single executable file that contains both the BoxLang VM engine and your compiled code. This uses an embedded minimal runner stub to ensure the final binary is ultra-lean (~500KB).

```bash
matchbox --target native my_script.bxs
```

## WebAssembly & Browser Support

`MatchBox` supports running BoxLang directly in the browser via WebAssembly.

### 1. Runtime Integration (JIT-like)
You can include the BoxLang engine in your page and run source code dynamically.

**Build the runtime:**
```bash
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir ./pkg target/wasm32-unknown-unknown/release/matchbox.wasm
```

**Use in HTML:**
```javascript
import init, { run_boxlang } from './pkg/matchbox.js';
await init();
run_boxlang('println("Hello World")');
```

### 2. Ahead-of-Time (AOT) Deployment
For production, you can compile your BoxLang code into a standalone WASM binary that contains your application bytecode in a custom section.

**Compile your app to WASM:**
```bash
# 1. Ensure runtime is built
cargo build --target wasm32-unknown-unknown --release

# 2. Compile your script to a specialized WASM binary
matchbox --target wasm my_app.bxs
# Produces: my_app.wasm
```

### 3. JavaScript Module Generation
You can compile BoxLang scripts into native JavaScript modules that run in the browser or Node.js via WASM:

```bash
matchbox --target js my_lib.bxs
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
| **Functions** | ✅ | `public numeric function add(a=1, b=2) { return a+b }` |
| **Strings** | ✅ | `"Hello #name#"`, `str1 & str2` |
| **Classes** | ✅ | `class MyClass extends="Base" accessors="true" { ... }` |
| **Interfaces**| ✅ | `interface I { function f(); }` |
| **Exceptions**| ✅ | `try { throw "err"; } catch(e) { ... }` |
| **Async** | ✅ | `f = runAsync(task); f.get(); sleep(100);` |
| **JS Interop**| ✅ | `js.window.location.href`, `js.alert("Hi")` |

## Technical Architecture

1. **Workspace Structure**: Divided into `matchbox-vm` (runtime), `matchbox-compiler` (frontend), and `matchbox-runner` (minimal stub).
2. **Parser**: Built using [Pest](https://pest.rs/) (PEG Grammar).
3. **Compiler**: Multi-stage compiler producing opcodes with line-number metadata.
4. **VM**: Stack-based machine with a cooperative fiber scheduler.
5. **Serialization**: Uses `bincode` for binary bytecode representation.
6. **Portability**: Native binaries are produced by appending bytecode to a pre-compiled architecture-specific runner stub.
