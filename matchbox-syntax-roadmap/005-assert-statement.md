# 005: Add `assert` Statement

**Type:** AFK  
**Blocked by:** #003 (Wire New Parser)

## What to build

Add `assert` statement support through all compiler layers.

BoxLang syntax:
```
assert isTrue;
assert isTrue : "optional message";
```

## Delivery

- **Parser:** Parse `assert` keyword → expression → optional `:` → message expression
- **AST:** Add `StatementKind::Assert { condition: Expression, message: Option<Expression> }` variant
- **Compiler:** Emit bytecode that evaluates the condition, and if falsy, throws an `AssertError` with the optional message string.
- **Test:** Integration test covering both forms (with and without message), passing and failing assertions.

## Acceptance criteria

- [ ] `assert expr;` parses and runs correctly
- [ ] `assert expr : "msg";` parses with optional message
- [ ] Failing assert throws an error
- [ ] Message expression is evaluated and included in error
- [ ] Integration test passes
