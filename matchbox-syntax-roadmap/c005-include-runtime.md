# C005: Include Statement Runtime

**Type:** AFK — Medium  
**Blocked by:** None

## Problem

The `include` statement compiles but only evaluates the path expression without loading or executing the file. For `include "path/to/file.bxs"`, the file is never loaded.

## Solution

### Phase 1: Static Includes (MVP)
For string literal includes like `include "path.bxs"`, the compiler can:
1. Detect that the argument is a `Literal::String` at compile time
2. Read the file content
3. Parse and compile it into a `Constant::CompiledFunction`
4. Emit a CALL opcode to execute it inline

### Phase 2: Dynamic Includes
For expression includes like `include variableName`, the VM needs:
1. An `INCLUDE` opcode that pops a path string
2. At runtime, reads the file, parses, compiles, and executes inline
3. Requires VM access to file system or embedded asset store
4. Variables set in included file must be visible to caller

### Compiler changes (Phase 1)
```rust
StatementKind::Include(expr) => {
    if let ExpressionKind::Literal(Literal::String(parts)) = &expr.kind {
        let path = parts.iter().map(|p| match p {
            StringPart::Text(t) => t.clone(),
            _ => String::new(),
        }).collect::<String>();
        if let Ok(source) = std::fs::read_to_string(&path) {
            // Parse and compile the included file
            let stmts = parser::parse(&source, Some(&path))?;
            let mut sub_compiler = Compiler::new(&path);
            // Compile into a sub-chunk and call it
            ...
        }
    }
    // Fall through: dynamic include (deferred)
    self.compile_expression(expr)?;
    self.chunk.emit0(op::POP, stmt.line as u32);
    Ok(())
}
```

### Test (Phase 1)
Helper file `tests/scripts/hello_include.bxs`:
```
var greeting = "Hello from include";
```

Test file:
```
include "tests/scripts/hello_include.bxs";
println(greeting); // expect "Hello from include"
```

## Acceptance criteria
- [ ] `include "file.bxs"` with string literal loads and executes at compile time
- [ ] Variables set in included file visible to caller
- [ ] Include works with both `.bxs` and `.bxm` files
- [ ] File not found produces compile error
- [ ] Integration test passes
