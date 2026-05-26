# Implement `QueryReduce` BIF

**Status:** Not Started  
**Category:** Query Transform  
**Priority:** Medium  

## BoxLang Reference
📁 `src/main/java/ortus/boxlang/runtime/bifs/global/query/QueryReduce.java`

## Description
Reduces query data to a single value by iteratively applying a callback function. The callback receives the accumulator and each row.

## Function Signature
```
QueryReduce(query, callback, initialValue)
```
| Argument       | Type     | Required | Description                                                          |
| -------------- | -------- | -------- | -------------------------------------------------------------------- |
| `query`        | Query    | ✅        | The query to reduce                                                  |
| `callback`     | Function | ✅        | Reducer receiving (accumulator, row, index, query) → new accumulator |
| `initialValue` | any      | ✅        | Initial accumulator value                                            |

**Returns:** `any` — the final accumulated value

## Implementation Notes
- Add `reduce` method on `BxQuery` in `crates/matchbox-vm/src/datasource/mod.rs`
- Register as `queryReduce` BIF in `crates/matchbox-vm/src/bifs/datasource.rs`
- Register in `crates/matchbox-vm/src/bifs/mod.rs` under the `bif-datasource` feature block
- Gate behind `bif-datasource` feature flag
- Callback receives `(accumulator, rowStruct, rowIndex, queryObject)`
- Iteratively apply callback: `acc = callback(acc, row, i, query)`

## Acceptance Criteria
- [ ] `query.reduce(callback, initialValue)` returns reduced value
- [ ] `queryReduce(myQuery, callback, initial)` works as standalone BIF
- [ ] Callback receives `(accumulator, rowStruct, rowIndex, queryObject)`
- [ ] Works with empty query (returns initialValue)
