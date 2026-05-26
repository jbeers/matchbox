# Implement `QuerySetRow` BIF

**Status:** Not Started  
**Category:** Query Mutation  
**Priority:** Medium  

## BoxLang Reference
📁 `src/main/java/ortus/boxlang/runtime/bifs/global/query/QuerySetRow.java`

## Description
Adds or updates a row in a query based on the provided row data and optional position. If no row number is provided, adds a new row at the end.

## Function Signature
```
QuerySetRow(query [, rowNumber, rowData])
```
| Argument    | Type             | Required | Default      | Description                  |
| ----------- | ---------------- | -------- | ------------ | ---------------------------- |
| `query`     | Query            | ✅        | —            | The query object             |
| `rowNumber` | Integer          | ❌        | `0` (append) | 1-based position for the row |
| `rowData`   | Struct/Array/any | ❌        | —            | The row data                 |

**Returns:** `Query` (the modified query)

## Implementation Notes
- Add `setRow` method on `BxQuery` in `crates/matchbox-vm/src/datasource/mod.rs`
- Register as `querySetRow` BIF in `crates/matchbox-vm/src/bifs/datasource.rs`
- Register in `crates/matchbox-vm/src/bifs/mod.rs` under the `bif-datasource` feature block
- Gate behind `bif-datasource` feature flag
- If rowNumber is 0 or omitted, append a new row
- If rowNumber is specified, update the existing row at that position
- Supports struct, array, or scalar row data (like `queryNew`)

## Acceptance Criteria
- [ ] `query.setRow(rowNumber=3, rowData=myStruct)` updates row 3
- [ ] `query.setRow(rowData=myStruct)` appends a new row
- [ ] `querySetRow(myQuery, 3, myStruct)` works as standalone BIF
- [ ] Supports struct, array, and scalar data types
- [ ] Returns the modified query
