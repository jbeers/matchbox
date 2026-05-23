# 010: Add `not` Expression-as-Statement

**Type:** AFK  
**Blocked by:** #003 (Wire New Parser)

## What to build

Add the `not` statement — a BoxLang quirk where `not` followed by an expression at the statement level is treated as `if (!expression)`.

BoxLang syntax:
```
not someCondition;
```

This is equivalent to `if (!someCondition) {}` — it evaluates the negated expression and does nothing with it. In practice, it's an assertion-like no-op at statement level. The important thing is that `not` followed by an expression is recognized as a statement and doesn't confuse the parser with `!` unary in expression context.

## Delivery

- **Parser:** In the statement parser, when `not` keyword is encountered at statement start, consume the following expression and produce a no-op statement (or desugar to `ExpressionStatement(UnaryNot(expr))`).
- **AST:** No new AST variant needed — can use existing `StatementKind::Expression` with a `UnaryNot` expression.
- **Compiler:** No changes needed — the expression compiles and is discarded.
- **Test:** Integration test verifying `not expr;` parses without error and doesn't affect program state.

## Acceptance criteria

- [ ] `not expr;` parses as a statement
- [ ] Does not interfere with `!expr` as a unary expression in other contexts
- [ ] Does not add runtime overhead beyond evaluating the expression
- [ ] Integration test passes
