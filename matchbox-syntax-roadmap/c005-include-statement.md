# C005: Implement include Statement

**Type:** AFK  
**Blocked by:** None

## What to build

Implement the `include` statement to load, parse, compile, and execute another BoxLang file at runtime.

### Compiler
- Compile the path expression (string literal or variable)
- Emit `INCLUDE` opcode

### VM opcode
`INCLUDE` (78) — pops path string from stack, loads file, parses it as BoxLang, compiles, and executes inline in the current VM context. Variables set in the included file should be visible to the including scope.

### Runtime
The VM needs access to the filesystem (or embedded asset store) to resolve the include path. In embedded/WASI contexts, the file may be pre-compiled.

### Test
```
// hello_include.bxs:
// var greeting = "Hello from include";

include "tests/scripts/hello_include.bxs";
println(greeting); // expect "Hello from include"
```

## Acceptance criteria
- [ ] `include "path/to/file.bxs"` loads and executes the file
- [ ] `include variableName` works with expression argument
- [ ] Variables set in included file are visible to caller
- [ ] Include works with both `.bxs` and `.bxm` files
- [ ] File not found produces runtime error
- [ ] Integration test passes
