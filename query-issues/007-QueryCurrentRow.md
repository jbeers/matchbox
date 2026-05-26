# Implement `QueryCurrentRow` BIF

**Status:** Not Started  
**Category:** Query Navigation  
**Priority:** Medium  

## BoxLang Reference
📁 `src/main/java/ortus/boxlang/runtime/bifs/global/query/QueryCurrentRow.java`

## Description
Returns the current row number (1-based) that the query cursor is positioned at. For matchbox, this requires adding a cursor/position tracker to `BxQuery`.

## Function Signature
```
QueryCurrentRow(query)
```
| Argument | Type  | Required | Description      |
| -------- | ----- | -------- | ---------------- |
| `query`  | Query | ✅        | The query object |

**Returns:** `Integer` — current row number (1-based), or 0 if at beginning

## Implementation Notes
- Add `current_row: usize` field to `BxQuery` in `crates/matchbox-vm/src/datasource/mod.rs`
- Add `currentRow` as a readable property on `BxQuery`
- Register as `queryCurrentRow` BIF in `crates/matchbox-vm/src/bifs/datasource.rs`
- Register in `crates/matchbox-vm/src/bifs/mod.rs` under the `bif-datasource` feature block
- Gate behind `bif-datasource` feature flag
- The cursor should start at 0 (before first row) and be updated as rows are iterated

## Acceptance Criteria
- [ ] `query.currentRow` returns 0 for a new query
- [ ] `queryCurrentRow(myQuery)` works as standalone BIF
- [ ] Cursor updates when iterating rows
