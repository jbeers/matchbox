# N024: QoQ Compact Row Storage

**Type:** Runtime / Performance
**Priority:** High

## What to build

Replace the HashMap-heavy QoQ row materialization path with a compact row
representation that keeps column values indexed and only exposes string-key
lookup where it is actually needed.

The goal is to cut both CPU and memory overhead on large in-memory queries,
especially the common aggregate and projection cases.

## Status

Implemented in the working tree.

The QoQ executor now materializes rows as indexed value vectors with a shared
column lookup map instead of per-row string-keyed maps. Release benchmark
coverage is recorded below. The full-script RSS still includes the original
`BxQuery` storage, so future memory profiling should isolate source query
storage from QoQ materialization if we need a finer-grained number.

## Acceptance criteria

- [x] QoQ can execute the existing join, WHERE, GROUP BY, HAVING, ORDER BY, and DISTINCT tests using the compact row path
- [x] The 1M-row AVG benchmark runs on the compact row path
- [x] Existing QoQ correctness tests still pass

## Verification

- `cargo test -p matchbox_compiler --features qoq`
- `cargo test --features "bif-datasource qoq" --test integration_tests vm_qoq`
- `target/release/matchbox benchmarks/qoq_avg_1m.bxs`
  - `build ms = 693`
  - `qoq ms = 57`
  - `Maximum resident set size = 198556 KB`
