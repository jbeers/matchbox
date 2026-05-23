# Review Issues From `feature/parser`

These issues come from the branch review against `develop` and the remaining-gap PRD.
They are focused on correctness, CPU performance, memory performance, and code quality.

## Issue Index

1. [R001: Special Operators Double-Compile Operands](r001-special-operators-double-compile.md)
2. [R002: `rethrow` Must Preserve The Active Exception](r002-rethrow-active-exception.md)
3. [R003: Implement Real `instanceof` And `castas` Semantics](r003-instanceof-castas-semantics.md)
4. [R004: Make Template Output And Script Islands Executable](r004-template-output-and-script-islands.md)
5. [R005: Support Case-Insensitive Keywords And Word Operators](r005-case-insensitive-keywords-word-operators.md)
6. [R006: Complete Spread Semantics](r006-complete-spread-semantics.md)
7. [R007: Fix Destructuring Evaluation Semantics](r007-destructuring-evaluation-semantics.md)
8. [R008: Preserve `assert` Message Semantics](r008-assert-message-semantics.md)
9. [R009: Preserve Modifier Metadata For Enforcement](r009-preserve-modifier-metadata.md)
10. [R010: Replace Eager Array Ranges With A Range Type](r010-range-type-lazy-semantics.md)
11. [R011: Finish The Lossless CST Tooling Surface](r011-finish-lossless-cst-tooling-surface.md)
12. [R012: Add Template Interpolation And Script-Island CST Boundaries](r012-template-cst-boundaries.md)

## Current CST Progress

The current branch now has a first-pass CST foundation:

- `cst::parse_script()` preserves source text, trivia, top-level statements, and nested braced block nodes.
- `cst::parse_template()` preserves exact template source by keeping source-gap spans around current template lexer tokens.
- The parser path now consumes span tokens from `lex()` / `lex_template()` instead of allocating an owned `String` lexeme for every token up front.

The remaining CST/tooling work is tracked in R011. The compiler/runtime semantic findings remain tracked in R001-R010.

R001, R002, R003, and R004 have been implemented in code and the corresponding issue files have been updated to reflect that status.

R005 has been implemented in code. Keyword lookup is case-insensitive, the common single-token aliases map to the canonical compiler forms, and the BoxLang phrase operators are covered by parser lookahead and compiler lowering.

R006 has been implemented in code. Function-call spread, struct spread, and invalid spread behavior are covered by runtime tests, and existing array spread behavior still passes.

R007 has been implemented in code. Destructuring now evaluates its source once, handles object and array patterns, and is covered by both runtime and script integration tests.

R008 has been implemented in code. `assert` now preserves optional custom messages, evaluates them only on failure, and still falls back to the default message when none is supplied.

R009 has been implemented in code. Class and function modifiers are now preserved in the AST and carried into compiled class/function metadata for future enforcement work.

R010 has been implemented in code. Range creation is now lazy, range iteration is handled without eager array materialization, and `len`/`contains` understand the range type.

R011 is partially implemented. The CST now has stable node ids, descendant traversal helpers, explicit error nodes for malformed braces, and typed kinds for common statements, but it still lacks expression-level structure and richer formatter-oriented metadata.

R012 has been implemented in code. Template CST now exposes structured interpolation and script-island nodes, preserves escaped hashes distinctly, and still round-trips exact source.
