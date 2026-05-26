# Implement `QueryAppend` BIF

**Status:** Not Started  
**Category:** Query Mutation  
**Priority:** High  

## BoxLang Reference
📁 `src/main/java/ortus/boxlang/runtime/bifs/global/query/QueryAppend.java`

## Description
Appends all rows from one query (`query2`) to the end of another query (`query1`). Both queries must have compatible column structures.

## Function Signature
```
QueryAppend(query1, query2)
```
| Argument | Type  | Required | Description                          |
| -------- | ----- | -------- | ------------------------------------ |
| `query1` | Query | ✅        | The destination query                |
| `query2` | Query | ✅        | The source query to append rows from |

**Returns:** `Query` (the modified `query1`)

## Implementation Notes
- Add `append` method on `BxQuery` in `crates/matchbox-vm/src/datasource/mod.rs`
- Register as `queryAppend` BIF in `crates/matchbox-vm/src/bifs/datasource.rs`
- Register in `crates/matchbox-vm/src/bifs/mod.rs` under the `bif-datasource` feature block
- Gate behind `bif-datasource` feature flag
- Only append rows from matching columns; ignore columns in query2 not present in query1

## Acceptance Criteria
- [ ] `query1.append(query2)` appends all rows from query2 to query1
- [ ] `queryAppend(q1, q2)` works as standalone BIF
- [ ] Only matching columns are appended; extra columns in source are ignored
- [ ] Missing columns in source get NULL values
- [ ] Returns the modified query1 (for chaining)
