# N016: QoQ Execution Engine

**Type:** Runtime / Execution
**Priority:** High

## What to build

Execute parsed QoQ queries against in-memory query data behind the optional
`qoq` feature flag.

## Status

Completed. The QoQ executor now resolves tables, joins, filters, grouping,
ordering, distinct, limit, unions, and subqueries against in-memory query data
behind `qoq`.

## Acceptance criteria

- [x] Table resolution works
- [x] JOIN/WHERE/GROUP BY/HAVING/ORDER BY work
- [x] DISTINCT, LIMIT, UNION, and subqueries work
- [x] Integration tests cover realistic query execution
