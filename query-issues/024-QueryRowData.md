# Implement `QueryRowData` BIF (and `getRow`)

**Status:** Not Started  
**Category:** Query Access  
**Priority:** High  

## BoxLang Reference
📁 `src/main/java/ortus/boxlang/runtime/bifs/global/query/QueryRowData.java`

## Description
Returns the cells of a query row as a struct. In BoxLang, this is exposed as both `queryRowData()` and the member method `getRow()`.

> **Note:** Matchbox already has `getRow()` as a native method on `BxQuery`, but does NOT expose `queryRowData()` as a standalone BIF.

## Function Signature
```
QueryRowData(query, rowNumber)
```
| Argument    | Type    | Required | Description       |
| ----------- | ------- | -------- | ----------------- |
| `query`     | Query   | ✅        | The query object  |
| `rowNumber` | Integer | ✅        | 1-based row index |

**Returns:** `Struct` — column name → value pairs

## Implementation Notes
- Register as `queryRowData` BIF in `crates/matchbox-vm/src/bifs/datasource.rs`
- Register in `crates/matchbox-vm/src/bifs/mod.rs` under the `bif-datasource` feature block
- Gate behind `bif-datasource` feature flag
- Simply delegates to the existing `getRow()` native method on `BxQuery`

## Acceptance Criteria
- [ ] `queryRowData(myQuery, 3)` returns struct of row 3 data
- [ ] Equivalent to `myQuery.getRow(3)`
- [ ] Returns empty struct for out-of-range row
