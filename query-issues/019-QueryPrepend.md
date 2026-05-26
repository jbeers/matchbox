# Implement `QueryPrepend` BIF

**Status:** Not Started  
**Category:** Query Mutation  
**Priority:** Medium  

## BoxLang Reference
📁 `src/main/java/ortus/boxlang/runtime/bifs/global/query/QueryPrepend.java`

## Description
Adds rows from one query to the **beginning** of another query. The inverse of `QueryAppend`.

## Function Signature
```
QueryPrepend(query1, query2)
```
| Argument | Type  | Required | Description                           |
| -------- | ----- | -------- | ------------------------------------- |
| `query1` | Query | ✅        | The destination query                 |
| `query2` | Query | ✅        | The source query to prepend rows from |

**Returns:** `Query` (the modified `query1`)

## Implementation Notes
- Add `prepend` method on `BxQuery` in `crates/matchbox-vm/src/datasource/mod.rs`
- Register as `queryPrepend` BIF in `crates/matchbox-vm/src/bifs/datasource.rs`
- Register in `crates/matchbox-vm/src/bifs/mod.rs` under the `bif-datasource` feature block
- Gate behind `bif-datasource` feature flag
- Insert rows at index 0; shift existing rows down
- Only matching columns are inserted

## Acceptance Criteria
- [ ] `query1.prepend(query2)` inserts query2 rows at the beginning of query1
- [ ] `queryPrepend(q1, q2)` works as standalone BIF
- [ ] Returns the modified query1
- [ ] Column matching works correctly
