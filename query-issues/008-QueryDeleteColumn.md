# Implement `QueryDeleteColumn` BIF

**Status:** Not Started  
**Category:** Query Mutation  
**Priority:** Medium  

## BoxLang Reference
📁 `src/main/java/ortus/boxlang/runtime/bifs/global/query/QueryDeleteColumn.java`

## Description
Deletes a column (by name) from a query object, removing all its data.

## Function Signature
```
QueryDeleteColumn(query, column)
```
| Argument | Type   | Required | Description               |
| -------- | ------ | -------- | ------------------------- |
| `query`  | Query  | ✅        | The query object          |
| `column` | String | ✅        | The column name to delete |

**Returns:** `Query` (the modified query)

## Implementation Notes
- Add `deleteColumn` method on `BxQuery` in `crates/matchbox-vm/src/datasource/mod.rs`
- Register as `queryDeleteColumn` BIF in `crates/matchbox-vm/src/bifs/datasource.rs`
- Register in `crates/matchbox-vm/src/bifs/mod.rs` under the `bif-datasource` feature block
- Gate behind `bif-datasource` feature flag
- Remove column from `columns` vec and corresponding data from `data` vec
- Case-insensitive column name matching

## Acceptance Criteria
- [ ] `query.deleteColumn("colName")` removes the column and its data
- [ ] `queryDeleteColumn(myQuery, "colName")` works as standalone BIF
- [ ] Case-insensitive column name matching
- [ ] Returns the modified query
