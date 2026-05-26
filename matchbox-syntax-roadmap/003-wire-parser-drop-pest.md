# 003: Wire New Parser, Drop Pest Dependency

**Type:** AFK  
**Blocked by:** #002 (Custom Parser)

## What to build

Replace the pest-based `BxParser` with the new hand-written lexer+parser. Remove `pest` and `pest_derive` from `Cargo.toml`. Ensure the public API is unchanged — `crate::parser::parse(source, filename) -> Result<Vec<Statement>>` remains the entry point.

## Delivery

Changes to:
- `crates/matchbox-compiler/Cargo.toml` — remove `pest`, `pest_derive`
- `crates/matchbox-compiler/src/parser/mod.rs` — replace `BxParser::parse(Rule::program, source)` with `tokenize()` → `parse_tokens()`
- Delete `crates/matchbox-compiler/src/parser/boxlang.pest`
- Remove `#[derive(Parser)]` and `#[grammar = "parser/boxlang.pest"]`
- Remove the `Rule` enum usage throughout the parser (the `Rule` enum was pest-generated)

The `parse_bxm()` function (behind `bxm` feature) should continue to work — it calls `parse()` internally, so it uses the new parser transitively.

## Acceptance criteria

- [ ] `pest` and `pest_derive` removed from Cargo.toml
- [ ] `boxlang.pest` file deleted
- [ ] `cargo build` succeeds without pest
- [ ] All 86+ integration tests pass (`cargo test`)
- [ ] `cargo test --features bxm` passes (if applicable)
- [ ] No `Rule::` or `pest::` imports remain in non-test code
- [ ] Public API signature unchanged
