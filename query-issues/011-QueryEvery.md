# Implement `QueryEvery` BIF

**Status:** Not Started  
**Category:** Query Predicate  
**Priority:** Medium  

## BoxLang Reference
📁 `src/main/java/ortus/boxlang/runtime/bifs/global/query/QueryEvery.java`

## Description
Tests whether **every** row in a query satisfies the predicate callback function. Returns `true` only if the callback returns truthy for all rows. Short-circuits on first `false`.

## Function Signature
```
QueryEvery(query, callback [, parallel, maxThreads, virtual])
```
| Argument     | Type     | Required | Default | Description                                       |
| ------------ | -------- | -------- | ------- | ------------------------------------------------- |
| `query`      | Query    | ✅        | —       | The query to test                                 |
| `callback`   | Function | ✅        | —       | Predicate receiving (row, index, query) → boolean |
| `parallel`   | Boolean  | ❌        | `false` | Execute in parallel                               |
| `maxThreads` | Integer  | ❌        | —       | Max threads for parallel mode                     |
| `virtual`    | Boolean  | ❌        | `false` | Use virtual threads                               |

**Returns:** `Boolean`

## Implementation Notes
- Add `every` method on `BxQuery` in `crates/matchbox-vm/src/datasource/mod.rs`
- Register as `queryEvery` BIF in `crates/matchbox-vm/src/bifs/datasource.rs`
- Register in `crates/matchbox-vm/src/bifs/mod.rs` under the `bif-datasource` feature block
- Gate behind `bif-datasource` feature flag
- Short-circuit: stop iterating on first `false` result
- Callback receives row struct, 1-based index, and query object

## Acceptance Criteria
- [ ] `query.every(callback)` returns `true` if all rows pass
- [ ] `queryEvery(myQuery, callback)` works as standalone BIF
- [ ] Short-circuits on first `false`
- [ ] Returns `true` for empty query
- [ ] Callback receives `(rowStruct, rowIndex, queryObject)`
