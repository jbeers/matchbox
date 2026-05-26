# N027: QoQ Query Source And Plan Fast Paths

**Type:** Runtime / Performance
**Priority:** High

## What to build

Add a QoQ table source abstraction so the executor can scan in-memory query data
directly instead of converting `BxQuery` into a row-major `QueryResult` and then
into QoQ rows.

This should become the foundation for the next set of QoQ performance work:
cached SQL plans, source-column dependency planning, and streaming aggregate
fast paths for simple single-table aggregate queries.

## Why

The current optimized QoQ executor is fast once rows are materialized, but the
large-query path still pays for duplicate representations of the same data. A
source/view abstraction lets QoQ read only the columns it needs, avoid row
materialization for aggregate scans, and cache repeated parse/planning work.

## Acceptance criteria

- [x] QoQ has a `QuerySource` / `TableView` style abstraction that can read source data by row and column without first building a full row-major `QueryResult`
- [x] `queryExecute(..., { dbType: "query" })` can execute existing QoQ tests through the source abstraction
- [x] Repeated identical QoQ SQL strings can reuse a parsed representation
- [x] The planner records the source columns required by projection, WHERE, GROUP BY, HAVING, ORDER BY, and aggregate expressions
- [x] Simple single-table aggregates such as `AVG(column)`, `SUM(column)`, `COUNT(column)`, `MIN(column)`, and `MAX(column)` can stream directly over the source column without materializing QoQ rows
- [x] The 1M-row AVG benchmark reports lower peak RSS than the current compact-row baseline
- [x] The 10k small-query benchmark improves or remains flat versus the current release baseline
- [x] Existing QoQ parser, executor, join, and BIF integration tests still pass

## Baseline

Current release benchmark numbers from the QoQ optimization checkpoint:

- `benchmarks/qoq_avg_1m.bxs`
  - MatchBox release: build `783ms`, QoQ `68ms`, wall `0.90s`, max RSS `198620 KB`
- `benchmarks/qoq_many_small_queries.bxs`
  - MatchBox release: elapsed `67ms`, wall `0.07s`, max RSS `20244 KB`

## Current implementation checkpoint

- Direct `BxQuery` scanning is wired through the VM native object interface.
- Generic QoQ materialization now reads through `QuerySource` instead of asking
  native query objects for a cloned row-major `QueryResult`.
- A bounded parse cache reuses parsed QoQ ASTs for repeated SQL strings.
- Simple single-table aggregate plans stream directly from source columns.
- Source-column dependency planning records projection, WHERE, GROUP BY, HAVING,
  ORDER BY, aggregate, and join predicate requirements.
- Generic materialization uses safe source-column plans to skip unused source
  columns when identifiers are unambiguous.
- `COUNT(*)` does not force star-style materialization of every source column.

Latest release benchmark numbers:

- `benchmarks/qoq_avg_1m.bxs`
  - MatchBox release: build `627ms`, QoQ `16ms`, wall `0.69s`, max RSS `144244 KB`
- `benchmarks/qoq_many_small_queries.bxs`
  - MatchBox release: elapsed `27ms`, wall `0.03s`, max RSS `20084 KB`

## Notes

- Keep the abstraction small enough to support direct `BxQuery` reads first.
- Do not optimize joins in this ticket beyond preserving current behavior.
- If the plan cache needs invalidation based on query shape, use column names and
  types rather than source object identity alone.
