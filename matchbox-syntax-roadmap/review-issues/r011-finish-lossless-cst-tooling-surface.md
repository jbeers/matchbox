# R011: Finish The Lossless CST Tooling Surface

**Type:** Tooling / Parser architecture / Memory performance  
**Priority:** Medium  
**Status:** Complete
**Related files:** `crates/matchbox-compiler/src/cst.rs`, `crates/matchbox-compiler/src/tokenizer.rs`, `crates/matchbox-compiler/tests/cst_tooling.rs`

## Current Status

Complete on the current branch.

Completed on the current branch:

- Added `SyntaxToken`, `Trivia`, `TriviaKind`, and `LexedSource`.
- Added `lex()` and `lex_template()` zero-copy span-token entry points.
- Kept the old `tokenize()` and `tokenize_template()` owned-lexeme APIs for compatibility.
- Routed script and template AST parsers through span tokens.
- Added `cst::parse_script()` with exact source round-trip.
- Added script CST nodes for top-level statements and nested braced blocks.
- Added `cst::parse_template()` with source-gap preservation.
- Added stable CST node ids and descendant traversal helpers.
- Added explicit CST error nodes for unmatched braces.
- Added typed CST node kinds for common statements such as variable declarations, returns, conditionals, loops, try/catch, function/class/interface declarations, and related control statements.
- Added shallow expression CST nodes for generic expression statements plus parenthesized, binary, and member-access forms.
- Added edge trivia helpers for formatter attachment policy.

## Problem

The CST is now useful enough to preserve source and identify broad script structure, but it is not yet complete enough to power a formatter or language server by itself.

Important missing pieces:

- Incremental parsing strategy is still a future extension.
- Template CST still needs richer structure beyond interpolation and script-island boundaries.

## Solution

Continue in vertical slices:

1. Incremental parsing can be added later if tooling requires it.
2. Template CST can be expanded further if formatter/LSP requirements grow.

## Test

Add integration-style tests through the public CST API:

```rust
let tree = matchbox_compiler::cst::parse_script("if (x) { return x; }");
assert_eq!(tree.to_source(), "if (x) { return x; }");
// Assert statement/block/expression nodes by public kind and source span.
```

## Acceptance Criteria

- [x] CST round-trips exact source for scripts and templates.
- [x] Common statement nodes have stable, queryable syntax kinds.
- [x] Common expression nodes have stable, queryable syntax kinds.
- [x] Comments and whitespace remain accessible as trivia.
- [x] Error nodes preserve malformed source without panicking.
- [x] Formatter/LSP consumers can traverse the tree without parsing token streams manually.
