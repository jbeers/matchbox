# Implement `QueryRowSwap` BIF

**Status:** Not Started  
**Category:** Query Mutation  
**Priority:** Medium  

## BoxLang Reference
📁 `src/main/java/ortus/boxlang/runtime/bifs/global/query/QueryRowSwap.java`

## Description
Swaps the positions of two rows in a query object. Both source and destination are 1-based row indices.

## Function Signature
```
QueryRowSwap(query, source, destination)
```
| Argument      | Type    | Required | Description                 |
| ------------- | ------- | -------- | --------------------------- |
| `query`       | Query   | ✅        | The query object            |
| `source`      | Integer | ✅        | 1-based index of first row  |
| `destination` | Integer | ✅        | 1-based index of second row |

**Returns:** `Query` (the modified query)

## Implementation Notes
- Add `rowSwap` method on `BxQuery` in `crates/matchbox-vm/src/datasource/mod.rs`
- Register as `queryRowSwap` BIF in `crates/matchbox-vm/src/bifs/datasource.rs`
- Register in `crates/matchbox-vm/src/bifs/mod.rs` under the `bif-datasource` feature block
- Gate behind `bif-datasource` feature flag
- Swap the values at both indices across all columns
- 1-based indexing

## Acceptance Criteria
- [ ] `query.rowSwap(2, 5)` swaps rows 2 and 5
- [ ] `queryRowSwap(myQuery, 2, 5)` works as standalone BIF
- [ ] All column data is swapped consistently
- [ ] Throws error for out-of-range indices
- [ ] Returns the modified query
