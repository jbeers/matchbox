# Implement `QuerySetCell` BIF

**Status:** Not Started  
**Category:** Query Mutation  
**Priority:** High  

## BoxLang Reference
📁 `src/main/java/ortus/boxlang/runtime/bifs/global/query/QuerySetCell.java`

## Description
Sets the value of a specific cell in a query by column name and optional row number. If no row number is provided, uses the current row.

## Function Signature
```
QuerySetCell(query, column, value [, row])
```
| Argument | Type    | Required | Default     | Description                    |
| -------- | ------- | -------- | ----------- | ------------------------------ |
| `query`  | Query   | ✅        | —           | The query object               |
| `column` | String  | ✅        | —           | Column name (case-insensitive) |
| `value`  | any     | ✅        | —           | The value to set               |
| `row`    | Integer | ❌        | current row | 1-based row index              |

**Returns:** `Query` (the modified query)

## Implementation Notes
- Add `setCell` method on `BxQuery` in `crates/matchbox-vm/src/datasource/mod.rs`
- Register as `querySetCell` BIF in `crates/matchbox-vm/src/bifs/datasource.rs`
- Register in `crates/matchbox-vm/src/bifs/mod.rs` under the `bif-datasource` feature block
- Gate behind `bif-datasource` feature flag
- Use `bx_to_sql` to convert `BxValue` to `SqlValue`
- Case-insensitive column name matching

## Acceptance Criteria
- [ ] `query.setCell("colName", newValue, 2)` sets the value at row 2
- [ ] `querySetCell(myQuery, "colName", newValue)` works as standalone BIF
- [ ] Defaults to current row if row omitted
- [ ] Case-insensitive column name matching
- [ ] Returns the modified query
