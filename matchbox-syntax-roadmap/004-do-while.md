# 004: Add `do/while` Loop Support

**Type:** AFK  
**Blocked by:** #003 (Wire New Parser)

## What to build

Add `do/while` loop support through all compiler layers: parser → AST → bytecode → VM execution.

BoxLang syntax:
```
do {
    statements;
} while (condition);
```

The body executes once before the condition is checked, then repeats while the condition is true.

## Delivery

- **Parser:** Parse `do` keyword → statement block → `while` keyword → `(` → expression → `)` → `;`
- **AST:** Add `StatementKind::DoWhile { body: Vec<Statement>, condition: Expression }` variant
- **Compiler:** Emit bytecode: unconditionally execute body, then test condition → jump back if true (or jump out if false). Structure: body → condition test → conditional backward jump.
- **Test:** Integration test `.bxs` file with basic `do/while`, empty body, multi-iteration, and nested `do/while`.

## Acceptance criteria

- [ ] `do { body; } while (cond);` parses correctly
- [ ] Body executes at least once even when condition is initially false
- [ ] Loops correctly while condition is true
- [ ] Works with `break` and `continue` inside body
- [ ] Nested `do/while` loops work correctly
- [ ] Integration test passes
