# Implement `QueryClear` BIF

**Status:** Not Started  
**Category:** Query Mutation  
**Priority:** High  

## BoxLang Reference
📁 `src/main/java/ortus/boxlang/runtime/bifs/global/query/QueryClear.java`

## Description
Clears all row data from a query object while preserving the column definitions.

## Function Signature
```
QueryClear(query)
```
| Argument | Type  | Required | Description               |
| -------- | ----- | -------- | ------------------------- |
| `query`  | Query | ✅        | The query object to clear |

**Returns:** `Query` (the cleared query)

## Implementation Notes
- Add `clear` method on `BxQuery` in `crates/matchbox-vm/src/datasource/mod.rs`
- Register as `queryClear` BIF in `crates/matchbox-vm/src/bifs/datasource.rs`
- Register in `crates/matchbox-vm/src/bifs/mod.rs` under the `bif-datasource` feature block
- Gate behind `bif-datasource` feature flag
- Must preserve column names and types; only clear row data and reset `recordCount` to 0

## Acceptance Criteria
- [ ] `query.clear()` removes all rows
- [ ] `queryClear(myQuery)` works as standalone BIF
- [ ] Column definitions are preserved after clear
- [ ] `recordCount` is reset to 0
- [ ] Returns the cleared query object
