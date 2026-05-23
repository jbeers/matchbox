# R011: Finish The Lossless CST Tooling Surface

**Type:** Tooling / Parser architecture / Memory performance  
**Priority:** Medium  
**Status:** Partially implemented
**Related files:** `crates/matchbox-compiler/src/cst.rs`, `crates/matchbox-compiler/src/tokenizer.rs`, `crates/matchbox-compiler/tests/cst_tooling.rs`

## Current Status

Partially implemented.

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

## Problem

The CST is now useful enough to preserve source and identify broad script structure, but it is not yet complete enough to power a formatter or language server by itself.

Important missing pieces:

- No expression-level CST nodes.
- No comment attachment policy for formatter use.
- No incremental parsing strategy.
- Template CST still needs richer structure beyond interpolation and script-island boundaries.

## Solution

Continue in vertical slices:

1. Add expression node grouping for parenthesized expressions, calls, member access, arrays, structs, and binary expression spans.
2. Add `SyntaxTree` traversal helpers that do not expose implementation details unnecessarily.
3. Add explicit error nodes for unterminated strings and unexpected EOF.
4. Add comment/trivia association helpers for formatter consumers.
5. Only then consider incremental parsing data structures.

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
- [ ] Common expression nodes have stable, queryable syntax kinds.
- [x] Comments and whitespace remain accessible as trivia.
- [x] Error nodes preserve malformed source without panicking.
- [x] Formatter/LSP consumers can traverse the tree without parsing token streams manually.
