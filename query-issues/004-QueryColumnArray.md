# Implement `QueryColumnArray` BIF

**Status:** Not Started  
**Category:** Query Introspection  
**Priority:** Medium  

## BoxLang Reference
📁 `src/main/java/ortus/boxlang/runtime/bifs/global/query/QueryColumnArray.java`

## Description
Returns the column names of a query as an array of strings.

## Function Signature
```
QueryColumnArray(query)
```
| Argument | Type  | Required | Description      |
| -------- | ----- | -------- | ---------------- |
| `query`  | Query | ✅        | The query object |

**Returns:** `Array` — array of column name strings

## Implementation Notes
- Add `columnArray` method on `BxQuery` in `crates/matchbox-vm/src/datasource/mod.rs`
- Register as `queryColumnArray` BIF in `crates/matchbox-vm/src/bifs/datasource.rs`
- Register in `crates/matchbox-vm/src/bifs/mod.rs` under the `bif-datasource` feature block
- Gate behind `bif-datasource` feature flag

## Acceptance Criteria
- [ ] `query.columnArray()` returns `["col1", "col2", ...]` as a BoxLang array
- [ ] `queryColumnArray(myQuery)` works as standalone BIF
- [ ] Returns empty array for query with no columns
