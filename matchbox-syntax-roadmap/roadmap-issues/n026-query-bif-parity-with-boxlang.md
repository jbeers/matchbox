# N026: Query BIF Parity With BoxLang

**Type:** Runtime / Compatibility
**Priority:** High

## What to build

Align MatchBox's query built-in functions with the BoxLang query API so the
same test scripts and examples behave the same way on both runtimes.

This includes the signatures, accepted argument shapes, and returned query
object behavior for the core query BIFs used by QoQ and in-memory query data.

## Status

Implemented for the current QoQ and datasource regression scripts.

`queryNew()` now accepts the BoxLang-style comma-delimited column/type forms and
the row-data shapes used by the current examples: arrays of arrays, arrays of
structs, and scalar arrays for one-column queries. Existing `queryAddRow()`,
`queryColumnData()`, and `queryColumnList()` coverage remains green.

## Acceptance criteria

- [x] `queryNew()` accepts the BoxLang forms used by current examples and tests
- [x] `queryAddRow()`, `queryColumnData()`, and `queryColumnList()` match BoxLang's observable behavior on the current regression script
- [x] Query BIF regression scripts pass without runtime-specific rewrites
- [x] Any remaining intentional divergences are documented explicitly

## Verification

- `cargo test --features "bif-datasource qoq" --test integration_tests datasource_query_new`
- `cargo test --features "bif-datasource qoq" --test integration_tests vm_qoq`
