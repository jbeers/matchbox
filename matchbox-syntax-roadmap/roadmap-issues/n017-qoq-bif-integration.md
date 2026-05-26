# N017: QoQ BIF Integration

**Type:** Runtime / Integration
**Priority:** Medium

## What to build

Wire QoQ into `queryExecute()` when the query uses BoxLang's in-memory query
database mode, behind the optional `qoq` feature flag.

## Completed

- `dbtype: "query"` routes to QoQ
- Positional and named params work
- Query results are returned in the expected shape

## Acceptance criteria

- [x] `dbtype: "query"` routes to QoQ
- [x] Positional and named params work
- [x] Query results are returned in the expected shape
