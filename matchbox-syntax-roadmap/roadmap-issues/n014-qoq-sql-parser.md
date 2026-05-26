# N014: SQL Parser For QoQ

**Type:** Parser
**Priority:** High

## What to build

Parse the QoQ SELECT grammar into SQL AST nodes behind the optional `qoq`
feature flag.

## Status

Completed. The parser now handles select/from/join/where/group by/having/order
by/limit, subqueries, union, case expressions, function calls, and source-aware
parse errors behind `qoq`.

## Acceptance criteria

- [x] SELECT/FROM/JOIN/WHERE/GROUP BY/HAVING/ORDER BY/LIMIT parse
- [x] Subqueries and UNION parse
- [x] CASE and function expressions parse
- [x] Parse errors include source location
