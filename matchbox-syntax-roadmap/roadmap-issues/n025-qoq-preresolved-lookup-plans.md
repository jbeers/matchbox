# N025: QoQ Pre-Resolved Lookup Plans

**Type:** Runtime / Performance
**Priority:** High

## What to build

Teach QoQ execution to pre-resolve repeated identifier and column lookups into
column indexes or access plans before iterating rows.

This should remove repeated string concatenation, repeated HashMap probing, and
repeated path resolution in hot loops such as projections, WHERE clauses,
ORDER BY, HAVING, and aggregates.

## Status

Implemented in the working tree.

The executor now pre-resolves repeated simple identifier access for projection,
ORDER BY, and common aggregate paths. Generic expression evaluation still uses
the regular evaluator, but simple identifier-heavy QoQ workloads no longer pay
the previous per-row string concatenation and repeated lookup cost in these hot
paths.

## Acceptance criteria

- [x] Simple aggregates like `AVG(column)` avoid repeated identifier resolution per row
- [x] ORDER BY and projection access use pre-resolved positions where possible
- [x] Repeated small-query benchmark is recorded for future regression checks
- [x] Existing QoQ parser and executor tests still pass

## Verification

- `cargo test -p matchbox_compiler --features qoq`
- `cargo test --features "bif-datasource qoq" --test integration_tests vm_qoq`
- `target/release/matchbox benchmarks/qoq_many_small_queries.bxs`
  - `elapsed ms = 93`
  - `Maximum resident set size = 20244 KB`
