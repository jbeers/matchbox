# Implement `QueryColumnExists` BIF

**Status:** Not Started  
**Category:** Query Introspection  
**Priority:** Medium  

## BoxLang Reference
📁 `src/main/java/ortus/boxlang/runtime/bifs/global/query/QueryColumnExists.java`

## Description
Checks whether a column with the given name exists in a query object. Case-insensitive.

## Function Signature
```
QueryColumnExists(query, column)
```
| Argument | Type   | Required | Description              |
| -------- | ------ | -------- | ------------------------ |
| `query`  | Query  | ✅        | The query object         |
| `column` | String | ✅        | The column name to check |

**Returns:** `Boolean`

## Implementation Notes
- Add `columnExists` method on `BxQuery` in `crates/matchbox-vm/src/datasource/mod.rs`
- Register as `queryColumnExists` BIF in `crates/matchbox-vm/src/bifs/datasource.rs`
- Register in `crates/matchbox-vm/src/bifs/mod.rs` under the `bif-datasource` feature block
- Gate behind `bif-datasource` feature flag
- Use existing `col_index()` pattern (case-insensitive) for the check

## Acceptance Criteria
- [ ] `query.columnExists("myCol")` returns `true`/`false`
- [ ] `queryColumnExists(myQuery, "myCol")` works as standalone BIF
- [ ] Case-insensitive matching
