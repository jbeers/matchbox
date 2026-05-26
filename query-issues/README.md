# Matchbox Query BIF Implementation Issues

This directory contains implementation issues for query Built-In Functions (BIFs) that exist in the BoxLang reference implementation but are **not yet implemented** in Matchbox.

## Reference

- **BoxLang source:** `~/dev/ortus-boxlang/BoxLang/src/main/java/ortus/boxlang/runtime/bifs/global/query/`
- **Matchbox BIFs:** `crates/matchbox-vm/src/bifs/datasource.rs`
- **Matchbox BIF registry:** `crates/matchbox-vm/src/bifs/mod.rs`
- **Matchbox BxQuery native object:** `crates/matchbox-vm/src/datasource/mod.rs`
- **Tests:** `tests/scripts/datasource_query_*.bxs` + `tests/integration_tests.rs`

---

## Current Parity: 25 of 35 query BIFs implemented (71%)

### ✅ Already Implemented

| BIF                 | Status     | Issue |
| ------------------- | ---------- | ----- |
| `queryExecute`      | ✅ Existing | —     |
| `queryNew`          | ✅ Existing | —     |
| `queryAddRow`       | ✅ Existing | —     |
| `queryColumnData`   | ✅ Existing | —     |
| `queryColumnList`   | ✅ Existing | —     |
| `queryClear`        | ✅ NEW      | #003  |
| `queryColumnArray`  | ✅ NEW      | #004  |
| `queryColumnCount`  | ✅ NEW      | #005  |
| `queryColumnExists` | ✅ NEW      | #006  |
| `queryKeyExists`    | ✅ NEW      | #016  |
| `queryRecordCount`  | ✅ NEW      | #020  |
| `queryRowData`      | ✅ NEW      | #024  |
| `queryReverse`      | ✅ NEW      | #023  |
| `queryRowSwap`      | ✅ NEW      | #025  |
| `querySlice`        | ✅ NEW      | #028  |
| `queryDeleteColumn` | ✅ NEW      | #008  |
| `queryDeleteRow`    | ✅ NEW      | #009  |
| `queryAddColumn`    | ✅ NEW      | #001  |
| `queryAppend`       | ✅ NEW      | #002  |
| `queryPrepend`      | ✅ NEW      | #019  |
| `queryGetCell`      | ✅ NEW      | #013  |
| `querySetCell`      | ✅ NEW      | #026  |
| `queryInsertAt`     | ✅ NEW      | #015  |
| `querySetRow`       | ✅ NEW      | #027  |
| `queryCurrentRow`   | ✅ NEW      | #007  |
| `queryGetResult`    | ✅ NEW      | #014  |

### ⚠️ Blocked — Requires `BxVM` Trait Changes

These BIFs need to invoke BoxLang closures from Rust. The `BxVM` trait currently has no `call_function_value` method. This must be added to the trait before these can be implemented:

| #   | Issue                             | Category  | Priority |
| --- | --------------------------------- | --------- | -------- |
| 010 | [QueryEach](010-QueryEach.md)     | Iteration | High     |
| 011 | [QueryEvery](011-QueryEvery.md)   | Predicate | Medium   |
| 012 | [QueryFilter](012-QueryFilter.md) | Transform | High     |
| 017 | [QueryMap](017-QueryMap.md)       | Transform | High     |
| 018 | [QueryNone](018-QueryNone.md)     | Predicate | Medium   |
| 021 | [QueryReduce](021-QueryReduce.md) | Transform | Medium   |
| 029 | [QuerySome](029-QuerySome.md)     | Predicate | Medium   |
| 030 | [QuerySort](030-QuerySort.md)     | Transform | High     |

### ⚠️ Special — Requires QoQ Integration

| #   | Issue                                                 | Category | Priority |
| --- | ----------------------------------------------------- | -------- | -------- |
| 022 | [QueryRegisterFunction](022-QueryRegisterFunction.md) | Utility  | Low      |

---

## Implementation Pattern

Each BIF follows this pattern:

1. **Add a method on `BxQuery`** in `crates/matchbox-vm/src/datasource/mod.rs`  
   → implement a new arm in `call_method`

2. **Register as a standalone BIF** in `crates/matchbox-vm/src/bifs/datasource.rs`  
   → create a `pub fn query_xxx(vm, args)` function that delegates to `vm.native_object_call_method()`

3. **Register in the BIF map** in `crates/matchbox-vm/src/bifs/mod.rs`  
   → add under the `#[cfg(feature = "bif-datasource")]` block

4. **Write `.bxs` test** in `tests/scripts/` and register in `tests/integration_tests.rs`

5. **Gate behind feature flag:** `#[cfg(feature = "bif-datasource")]`

---

## Blocking Architectural Change

To implement the remaining 8 closure-based BIFs, the following addition to the `BxVM` trait is needed:

```rust
// In crates/matchbox-vm/src/types/mod.rs — BxVM trait
fn call_function_value(&mut self, func: BxValue, args: Vec<BxValue>) -> Result<BxValue, String>;
```

The concrete VM (`crates/matchbox-vm/src/vm/mod.rs`) already has this method. It just needs to be exposed through the trait.

