# 009: Add `include` Statement

**Type:** AFK  
**Blocked by:** #003 (Wire New Parser)

## What to build

Add `include` statement support through all compiler layers.

BoxLang syntax:
```
include "template.bxm";
include variableContainingPath;
```

`include` loads and executes another BoxLang source file at runtime. The argument is an expression evaluating to a file path string.

## Delivery

- **Parser:** Parse `include` keyword → expression → `;`
- **AST:** Add `StatementKind::Include(Expression)` variant
- **Compiler:** Emit bytecode that evaluates the path expression at runtime and invokes the `include` runtime operation (which loads, parses, compiles, and executes the file inline).
- **Test:** Integration test with a helper `.bxs` file that is included and produces observable side effects.

## Acceptance criteria

- [ ] `include "path/to/file.bxm";` parses correctly
- [ ] `include variableName;` parses with expression argument
- [ ] Included file is executed at runtime
- [ ] Variables set in included file are visible to the including scope
- [ ] Integration test passes
