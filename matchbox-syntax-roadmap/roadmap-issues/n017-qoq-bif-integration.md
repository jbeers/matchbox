# N017: QoQ BIF Integration

**Type:** Runtime / Integration
**Priority:** Medium

## What to build

Wire QoQ into `queryExecute()` when the query uses BoxLang's in-memory query
database mode.

## Acceptance criteria

- [ ] `dbtype: "query"` routes to QoQ
- [ ] Positional and named params work
- [ ] Query results are returned in the expected shape

