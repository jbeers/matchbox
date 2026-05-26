# N015: SQL AST For QoQ

**Type:** AST / Parser architecture
**Priority:** High

## What to build

Define the SQL AST types that the QoQ parser and runtime will share behind the
optional `qoq` feature flag.

## Status

Completed. The QoQ AST now covers select, table, join, union, expressions, and
literals, preserves the root query span, and exposes traversal helpers for the
executor.

## Acceptance criteria

- [x] AST covers select, table, join, union, expressions, and literals
- [x] Source spans are preserved
- [x] Traversal helpers exist for the executor
