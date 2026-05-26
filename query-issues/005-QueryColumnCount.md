# Implement `QueryColumnCount` BIF

**Status:** Not Started  
**Category:** Query Introspection  
**Priority:** Medium  

## BoxLang Reference
📁 `src/main/java/ortus/boxlang/runtime/bifs/global/query/QueryColumnCount.java`

## Description
Returns the number of columns in a query object.

## Function Signature
```
QueryColumnCount(query)
```
| Argument | Type  | Required | Description      |
| -------- | ----- | -------- | ---------------- |
| `query`  | Query | ✅        | The query object |

**Returns:** `Integer` — number of columns

## Implementation Notes
- Add `columnCount` method on `BxQuery` in `crates/matchbox-vm/src/datasource/mod.rs`
- Alternatively, add as a property on `BxQuery` (like `recordCount`)
- Register as `queryColumnCount` BIF in `crates/matchbox-vm/src/bifs/datasource.rs`
- Register in `crates/matchbox-vm/src/bifs/mod.rs` under the `bif-datasource` feature block
- Gate behind `bif-datasource` feature flag

## Acceptance Criteria
- [ ] `query.columnCount` returns the number of columns
- [ ] `queryColumnCount(myQuery)` works as standalone BIF
- [ ] Returns `0` for query with no columns
