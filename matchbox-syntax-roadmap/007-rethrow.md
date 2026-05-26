# 007: Add `rethrow` Statement

**Type:** AFK  
**Blocked by:** #003 (Wire New Parser)

## What to build

Add `rethrow` statement support through all compiler layers.

BoxLang syntax:
```
catch (e) {
    // handle...
    rethrow;
}
```

`rethrow` re-throws the currently caught exception, preserving the original stack trace. Only valid inside a `catch` block.

## Delivery

- **Parser:** Parse `rethrow` keyword → `;`
- **AST:** Add `StatementKind::Rethrow` variant
- **Compiler:** Validate `rethrow` is inside a `catch` block (compile error otherwise). Emit bytecode that re-throws the current exception from the catch context.
- **Test:** Integration test verifying rethrow inside catch works, and rethrow outside catch produces a compile error.

## Acceptance criteria

- [ ] `rethrow;` parses correctly
- [ ] Inside `catch`, `rethrow` re-throws the caught exception
- [ ] Outside `catch`, `rethrow` produces a compile error
- [ ] Original stack trace is preserved (if VM supports it)
- [ ] Integration test passes
