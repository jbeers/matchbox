# T005: Template Parser Foundation — Literal Text, Output, and Set

**Type:** AFK  
**Blocked by:** T001-T004 (All lexer modes)

## What to build

Build the template parser on top of the template tokenizer. Parse literal text content, `<bx:output>` blocks, and `<bx:set>` tags. Replace the old `bxm.rs` regex transpiler with direct template parsing.

### Template parser entry point
- New `parse_template(source, filename) -> Result<Vec<Statement>>` function
- Calls template-mode tokenizer, then template statement parser
- Template statements include: literal text, `<bx:output>`, `<bx:set>`, `<bx:if>`, etc.

### Literal text (CONTENT_TEXT)
- Emits `StatementKind::BufferOutput(Expression)` where expression is a `String` literal
- Adjacent ContentText tokens are merged for efficiency

### `<bx:output>` handling
- Push TEMPLATE_OUTPUT_MODE
- Body content parsed as template statements
- `#expr#` inside body: parse expression, wrap in StringInterpolation, emit as BufferOutput
- Closing `</bx:output>` pops output mode

### `<bx:set>` handling
- Parse attributes as assignment expression
- `<bx:set x = 10>` → `ExpressionStatement(Assignment(x, 10))`
- `<bx:set x = foo() />` (self-closing)

### New/Extended AST
- `StatementKind::BufferOutput(Expression)` — write to output buffer
- Reuse existing `Literal::String(Vec<StringPart>)` for string interpolation
- Existing `ExpressionKind::Assignment` for set

## Delivery

New files: `crates/matchbox-compiler/src/parser/template.rs`
Changes: `crates/matchbox-compiler/src/ast/mod.rs` for BufferOutput variant

## Acceptance criteria

- [ ] Literal HTML text is parsed as BufferOutput with String literal
- [ ] `<bx:output>hello #name#</bx:output>` produces BufferOutput with StringInterpolation
- [ ] `<bx:set x = 10>` produces ExpressionStatement with Assignment
- [ ] `<bx:set x = expr />` self-closing works
- [ ] Multiple ContentText tokens are merged
- [ ] Existing `.bxm` integration tests pass with new parser
- [ ] `bxm.rs` transpiler removed or deprecated
