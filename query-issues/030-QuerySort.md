# Implement `QuerySort` BIF

**Status:** Not Started  
**Category:** Query Transform  
**Priority:** High  

## BoxLang Reference
📁 `src/main/java/ortus/boxlang/runtime/bifs/global/query/QuerySort.java`

## Description
Sorts query rows using a comparator callback function. The callback receives two rows and should return a comparison result (-1, 0, 1).

## Function Signature
```
QuerySort(query, sortFunc)
```
| Argument   | Type     | Required | Description                                |
| ---------- | -------- | -------- | ------------------------------------------ |
| `query`    | Query    | ✅        | The query to sort                          |
| `sortFunc` | Function | ✅        | Comparator receiving (row1, row2) → -1/0/1 |

**Returns:** `Query` (the sorted query, mutated in-place)

## Implementation Notes
- Add `sort` method on `BxQuery` in `crates/matchbox-vm/src/datasource/mod.rs`
- Register as `querySort` BIF in `crates/matchbox-vm/src/bifs/datasource.rs`
- Register in `crates/matchbox-vm/src/bifs/mod.rs` under the `bif-datasource` feature block
- Gate behind `bif-datasource` feature flag
- Sort in-place (mutate the query)
- The comparator callback receives two row structs and returns negative/zero/positive
- Reorder all column data arrays consistently based on sort order

## Acceptance Criteria
- [ ] `query.sort(comparator)` sorts rows in-place
- [ ] `querySort(myQuery, comparator)` works as standalone BIF
- [ ] All column data remains consistent after sort
- [ ] Comparator receives `(rowAStruct, rowBStruct)`
- [ ] Returns the sorted query
- [ ] Stable sort: equal elements maintain relative order
