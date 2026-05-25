# N013: SQL Lexer For QoQ

**Type:** Parser / Lexer
**Priority:** High

## What to build

Add a dedicated SQL lexer for Query-of-Queries behind the optional `qoq`
feature flag.

## Status

Completed. The lexer is feature-gated behind `qoq` and now tokenizes the BoxLang
QoQ surface with case-insensitive keywords, literal spans, bind parameters, and
ODBC date/time escapes.

## Acceptance criteria

- [x] Keywords are case-insensitive
- [x] Strings, numbers, identifiers, comments, and bind params tokenize correctly
- [x] ODBC date/time literals are recognized
