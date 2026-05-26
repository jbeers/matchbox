# Implement `QueryReverse` BIF

**Status:** Not Started  
**Category:** Query Transform  
**Priority:** Medium  

## BoxLang Reference
📁 `src/main/java/ortus/boxlang/runtime/bifs/global/query/QueryReverse.java`

## Description
Reverses the order of rows in a query object. Mutates the query in-place.

## Function Signature
```
QueryReverse(query)
```
| Argument | Type  | Required | Description          |
| -------- | ----- | -------- | -------------------- |
| `query`  | Query | ✅        | The query to reverse |

**Returns:** `Query` (the reversed query)

## Implementation Notes
- Add `reverse` method on `BxQuery` in `crates/matchbox-vm/src/datasource/mod.rs`
- Register as `queryReverse` BIF in `crates/matchbox-vm/src/bifs/datasource.rs`
- Register in `crates/matchbox-vm/src/bifs/mod.rs` under the `bif-datasource` feature block
- Gate behind `bif-datasource` feature flag
- Reverse each column's data vec in-place

## Acceptance Criteria
- [ ] `query.reverse()` reverses row order in-place
- [ ] `queryReverse(myQuery)` works as standalone BIF
- [ ] All column data is reversed consistently
- [ ] Returns the modified query
