# N015: SQL AST For QoQ

**Type:** AST / Parser architecture
**Priority:** High

## What to build

Define the SQL AST types that the QoQ parser and runtime will share.

## Acceptance criteria

- [ ] AST covers select, table, join, union, expressions, and literals
- [ ] Source spans are preserved
- [ ] Traversal helpers exist for the executor

