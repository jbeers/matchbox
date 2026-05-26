# Implement `QueryDeleteRow` BIF

**Status:** Not Started  
**Category:** Query Mutation  
**Priority:** Medium  

## BoxLang Reference
📁 `src/main/java/ortus/boxlang/runtime/bifs/global/query/QueryDeleteRow.java`

## Description
Deletes a row from a query object at the specified 1-based index.

## Function Signature
```
QueryDeleteRow(query, row)
```
| Argument | Type    | Required | Description                 |
| -------- | ------- | -------- | --------------------------- |
| `query`  | Query   | ✅        | The query object            |
| `row`    | Integer | ✅        | 1-based row index to delete |

**Returns:** `Query` (the modified query)

## Implementation Notes
- Add `deleteRow` method on `BxQuery` in `crates/matchbox-vm/src/datasource/mod.rs`
- Register as `queryDeleteRow` BIF in `crates/matchbox-vm/src/bifs/datasource.rs`
- Register in `crates/matchbox-vm/src/bifs/mod.rs` under the `bif-datasource` feature block
- Gate behind `bif-datasource` feature flag
- Remove the row at the given 1-based index from each column's data vec
- Decrement `recordCount`

## Acceptance Criteria
- [ ] `query.deleteRow(3)` removes the 3rd row from all columns
- [ ] `queryDeleteRow(myQuery, 3)` works as standalone BIF
- [ ] Throws error for out-of-range row index
- [ ] `recordCount` is decremented correctly
- [ ] Returns the modified query
