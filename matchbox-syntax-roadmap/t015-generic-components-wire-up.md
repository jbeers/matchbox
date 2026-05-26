# T015: Generic Components + Wire Up, Drop BXM Transpiler

**Type:** AFK  
**Blocked by:** T005-T014 (All template parser features)

## What to build

Support generic `<bx:custom>` component tags and wire the new template parser into the compilation pipeline. Remove the old `bxm.rs` regex transpiler.

### Generic components
- Any `<bx:name attr="val"> body </bx:name>` that doesn't match a known keyword
- Produce `StatementKind::Component { name, attributes, body }`
- Attributes: `Vec<(String, Expression)>` — name + value pairs
- Body: `Vec<Statement>` — template statements
- Self-closing: `<bx:name attr="val" />`

### Wire up
- Replace `parse_bxm()` to use the new template parser instead of the regex transpiler
- Template entry point: `parse_template(source, filename)` returns `Vec<Statement>`
- Detected by `.bxm` extension
- Remove `bxm.rs` and the `regex` optional dependency

### Compiler integration
- Compile `BufferOutput` statements
- Compile `Component` as runtime component invocation
- Compile `ScriptIsland` and `TemplateIsland` as inline statements

## Delivery

Remove: `crates/matchbox-compiler/src/parser/bxm.rs`
Changes: `crates/matchbox-compiler/src/parser/mod.rs`, `crates/matchbox-compiler/src/compiler/mod.rs`

## Acceptance criteria

- [ ] Generic `<bx:custom>` tags parse without error
- [ ] Attributes with `name="value"` and `name=#expr#` parse
- [ ] Self-closing components parse
- [ ] `.bxm` files route through new template parser
- [ ] `bxm.rs` deleted, `regex` dependency removed
- [ ] All existing template integration tests pass
- [ ] Template tests from T005-T014 all pass
