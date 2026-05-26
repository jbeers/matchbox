# Implement `QuerySlice` BIF

**Status:** Not Started  
**Category:** Query Transform  
**Priority:** Medium  

## BoxLang Reference
📁 `src/main/java/ortus/boxlang/runtime/bifs/global/query/QuerySlice.java`

## Description
Returns a subset of rows from an existing query, starting at a given offset and optionally limited to a specific length.

## Function Signature
```
QuerySlice(query, offset [, length])
```
| Argument | Type    | Required | Default      | Description                |
| -------- | ------- | -------- | ------------ | -------------------------- |
| `query`  | Query   | ✅        | —            | The source query           |
| `offset` | Integer | ✅        | —            | 1-based starting row index |
| `length` | Integer | ❌        | `0` (to end) | Number of rows to include  |

**Returns:** `Query` — a new query with the sliced rows

## Implementation Notes
- Add `slice` method on `BxQuery` in `crates/matchbox-vm/src/datasource/mod.rs`
- Register as `querySlice` BIF in `crates/matchbox-vm/src/bifs/datasource.rs`
- Register in `crates/matchbox-vm/src/bifs/mod.rs` under the `bif-datasource` feature block
- Gate behind `bif-datasource` feature flag
- Returns a **new** query object
- 1-based offset
- If length is 0, include all rows from offset to end
- Preserves column definitions

## Acceptance Criteria
- [ ] `query.slice(2, 5)` returns new query with rows 2-6
- [ ] `query.slice(3)` returns new query with rows 3 to end
- [ ] `querySlice(myQuery, 2, 5)` works as standalone BIF
- [ ] Original query is not modified
- [ ] Column definitions are preserved
