# Implement `QueryAddColumn` BIF

**Status:** Not Started  
**Category:** Query Mutation  
**Priority:** High  

## BoxLang Reference
📁 `src/main/java/ortus/boxlang/runtime/bifs/global/query/QueryAddColumn.java`

## Description
Adds a new column to a query object and optionally populates its rows with the contents of a one-dimensional array.

## Function Signature
```
QueryAddColumn(query, columnName [, datatype, array])
```
| Argument     | Type   | Required | Default    | Description                            |
| ------------ | ------ | -------- | ---------- | -------------------------------------- |
| `query`      | Query  | ✅        | —          | The query object to add a column to    |
| `columnName` | String | ✅        | —          | Name of the new column                 |
| `datatype`   | String | ❌        | `"object"` | Data type for the column               |
| `array`      | Array  | ❌        | `[]`       | Array of values to populate the column |

**Returns:** `Query` (the modified query)

## Implementation Notes
- Add `add_column` method on `BxQuery` in `crates/matchbox-vm/src/datasource/mod.rs`
- Register as `queryAddColumn` BIF in `crates/matchbox-vm/src/bifs/datasource.rs`
- Register in `crates/matchbox-vm/src/bifs/mod.rs` under the `bif-datasource` feature block
- Gate behind `bif-datasource` feature flag
- Support column type parsing via existing `parse_col_type()` helper

## Acceptance Criteria
- [ ] `query.addColumn("newCol")` adds a new column with NULL/empty values for all existing rows
- [ ] `query.addColumn("newCol", "integer", [1,2,3])` adds column with typed data
- [ ] `queryAddColumn(myQuery, "newCol")` works as standalone BIF
- [ ] Column type parsing matches BoxLang behavior
- [ ] Returns the modified query object (for chaining)
