# Implement `QueryFilter` BIF

**Status:** Not Started  
**Category:** Query Transform  
**Priority:** High  

## BoxLang Reference
📁 `src/main/java/ortus/boxlang/runtime/bifs/global/query/QueryFilter.java`

## Description
Returns a new query containing only the rows that pass the predicate callback function.

## Function Signature
```
QueryFilter(query, callback [, parallel, maxThreads, virtual])
```
| Argument     | Type     | Required | Default | Description                                       |
| ------------ | -------- | -------- | ------- | ------------------------------------------------- |
| `query`      | Query    | ✅        | —       | The query to filter                               |
| `callback`   | Function | ✅        | —       | Predicate receiving (row, index, query) → boolean |
| `parallel`   | Boolean  | ❌        | `false` | Execute in parallel                               |
| `maxThreads` | Integer  | ❌        | —       | Max threads for parallel mode                     |
| `virtual`    | Boolean  | ❌        | `false` | Use virtual threads                               |

**Returns:** `Query` — a new query with matching rows

## Implementation Notes
- Add `filter` method on `BxQuery` in `crates/matchbox-vm/src/datasource/mod.rs`
- Register as `queryFilter` BIF in `crates/matchbox-vm/src/bifs/datasource.rs`
- Register in `crates/matchbox-vm/src/bifs/mod.rs` under the `bif-datasource` feature block
- Gate behind `bif-datasource` feature flag
- Returns a **new** query object (not mutating the original)
- Preserves column definitions and types from the source query
- Callback receives row struct, 1-based index, and query object

## Acceptance Criteria
- [ ] `query.filter(callback)` returns new query with matching rows
- [ ] `queryFilter(myQuery, callback)` works as standalone BIF
- [ ] Original query is not modified
- [ ] Column definitions are preserved
- [ ] Callback receives `(rowStruct, rowIndex, queryObject)`
