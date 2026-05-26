# Implement `QueryGetCell` BIF

**Status:** Not Started  
**Category:** Query Access  
**Priority:** High  

## BoxLang Reference
📁 `src/main/java/ortus/boxlang/runtime/bifs/global/query/QueryGetCell.java`

## Description
Gets the value of a specific cell in a query by column name and row number (1-based). If no row number is provided, uses the current row.

## Function Signature
```
QueryGetCell(query, columnName [, rowNumber])
```
| Argument     | Type    | Required | Default     | Description                    |
| ------------ | ------- | -------- | ----------- | ------------------------------ |
| `query`      | Query   | ✅        | —           | The query object               |
| `columnName` | String  | ✅        | —           | Column name (case-insensitive) |
| `rowNumber`  | Integer | ❌        | current row | 1-based row index              |

**Returns:** `any` — the cell value

## Implementation Notes
- Add `getCell` method on `BxQuery` in `crates/matchbox-vm/src/datasource/mod.rs`
- Register as `queryGetCell` BIF in `crates/matchbox-vm/src/bifs/datasource.rs`
- Register in `crates/matchbox-vm/src/bifs/mod.rs` under the `bif-datasource` feature block
- Gate behind `bif-datasource` feature flag
- Use `sql_to_bx` to convert `SqlValue` back to `BxValue`
- Case-insensitive column name matching
- Throws error for invalid column or out-of-range row

## Acceptance Criteria
- [ ] `query.getCell("colName", 2)` returns the value at row 2, column "colName"
- [ ] `queryGetCell(myQuery, "colName", 2)` works as standalone BIF
- [ ] Defaults to current row if rowNumber omitted
- [ ] Case-insensitive column name matching
- [ ] Throws for invalid column or row
