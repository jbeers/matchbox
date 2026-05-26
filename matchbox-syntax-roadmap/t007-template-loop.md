# T007: Template Loop

**Type:** AFK  
**Blocked by:** T005 (Template parser foundation)

## What to build

Parse `<bx:loop>` tags into `StatementKind::ForLoop` AST nodes.

### Syntax
BoxLang supports multiple loop forms:
```
<!-- for-in with array -->
<bx:loop array="#myArray#" item="item" [index="idx"]>
    body
</bx:loop>

<!-- for-in with struct -->
<bx:loop collection="#myStruct#" item="value" [key="k"]>
    body
</bx:loop>

<!-- C-style for -->
<bx:loop from="1" to="10" [step="2"] index="i">
    body
</bx:loop>

<!-- while-style -->
<bx:loop condition="i LT 10">
    body
</bx:loop>

<!-- query loop -->
<bx:loop query="myQuery">
    body
</bx:loop>
```

### Parsing
- Parse attributes: `array`, `collection`, `item`, `index`, `from`, `to`, `step`, `condition`, `query`
- Map to appropriate AST: `ForLoop` (for-in) or `WhileLoop` (condition-based)
- Body is template statements

### Simplification for MVP
Focus on the two most common forms:
1. `array="#expr#" item="name" [index="name"]` → `ForLoop`
2. `condition="expr"` (as string or `#expr#`) → parse expression from string → `WhileLoop`

## Delivery

Changes: `crates/matchbox-compiler/src/parser/template.rs`

## Acceptance criteria

- [ ] `<bx:loop array="#arr#" item="val"> body </bx:loop>` parses as ForLoop
- [ ] Optional `index` attribute produces index variable
- [ ] `<bx:loop from="1" to="10" index="i"> body </bx:loop>` parses correctly
- [ ] `<bx:loop condition="i LT 10"> body </bx:loop>` parses as WhileLoop
- [ ] Attribute expressions with `#expr#` are evaluated
- [ ] Integration test: `.bxm` with loop tags
