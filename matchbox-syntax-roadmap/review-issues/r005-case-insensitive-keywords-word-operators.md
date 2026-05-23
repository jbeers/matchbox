# R005: Support Case-Insensitive Keywords And Word Operators

**Type:** Correctness / Parser compatibility  
**Priority:** Medium  
**Related files:** `crates/matchbox-compiler/src/tokenizer.rs`, `crates/matchbox-compiler/src/parser/mod.rs`
**Reference:** `~/dev/ortus-boxlang/BoxLang/src/main/antlr/BoxLexer.g4`, `~/dev/ortus-boxlang/BoxLang/src/main/antlr/BoxGrammar.g4`

## Problem

The new lexer matches keywords and word operators case-sensitively. The BoxLang JVM lexer uses case-insensitive matching.

Common BoxLang forms are therefore missed:

- `FUNCTION`, `IF`, `TRUE`, `FALSE`
- `AND`, `OR`
- `EQ`, `GT`, `GTE`, `LT`, `LTE`, `NEQ`
- `IS`, `NOT CONTAINS`, `LESS THAN`, `GREATER THAN`

This is visible in template tests such as `x GT 5`, which parse successfully only because the condition parser falls back to a default `true` condition.

## Current Status

Implemented in code:

- keyword lookup is now case-insensitive
- boolean word operators and relational aliases like `AND`, `OR`, `EQ`, `GT`, `GTE`, `LT`, `LTE`, and `NEQ` now parse through the native compiler path
- multi-token phrase operators such as `NOT CONTAINS`, `LESS THAN`, `GREATER THAN`, `DOES NOT CONTAIN`, `IS NOT`, and `... OR EQUAL TO` forms are also supported

## Solution

Normalize keyword lookup without changing the stored lexeme.

Suggested approach:

1. Use `lexeme.to_ascii_lowercase()` in `keyword_or_ident`.
2. Add token kinds for BoxLang word operators and aliases.
3. Teach the Pratt parser to map those token kinds to the canonical operator strings.
4. Add multi-token relational phrases if full compatibility is in scope.

## Test

```boxlang
IF (TRUE AND 3 GT 2) {
    println("ok");
}
```

## Acceptance Criteria

- [x] Keywords tokenize case-insensitively.
- [x] Boolean word operators `AND` and `OR` work.
- [x] Relational aliases `EQ`, `GT`, `GTE`, `LT`, `LTE`, and `NEQ` work.
- [x] Existing lower-case syntax still works.
- [x] Template condition tests assert branch behavior, not only parse success.
