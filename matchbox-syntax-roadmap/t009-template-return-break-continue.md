# T009: Template return + break/continue

**Type:** AFK  
**Blocked by:** T005 (Template parser foundation)

## What to build

Parse `<bx:return>`, `<bx:break>`, and `<bx:continue>` tags.

### Syntax
```
<bx:return [expression]>
<bx:return expression />
<bx:break [label]>
<bx:break />
<bx:continue [label]>
<bx:continue />
```

### Parsing
- `<bx:return>`: Parse optional expression, produce `StatementKind::Return(Option<Expression>)`
- `<bx:break>`: Parse optional label, produce `StatementKind::Break`
- `<bx:continue>`: Parse optional label, produce `StatementKind::Continue`
- Expression in `<bx:return>` is parsed as script expression
- Self-closing forms (`/>`) supported

### Validation
- `<bx:break>` and `<bx:continue>` are only valid inside `<bx:loop>` or `<bx:while>`
- Report parse error if used outside loop context

## Delivery

Changes: `crates/matchbox-compiler/src/parser/template.rs`

## Acceptance criteria

- [ ] `<bx:return>` parses with no expression
- [ ] `<bx:return expr>` parses expression
- [ ] `<bx:return expr />` self-closing works
- [ ] `<bx:break>` and `<bx:continue>` parse correctly
- [ ] Labels on break/continue are parsed
- [ ] Integration test: `.bxm` with return/break/continue
