# Implement `QuerySome` BIF

**Status:** Not Started  
**Category:** Query Predicate  
**Priority:** Medium  

## BoxLang Reference
📁 `src/main/java/ortus/boxlang/runtime/bifs/global/query/QuerySome.java`

## Description
Tests whether **at least one** row in a query satisfies the predicate callback function. Returns `true` if the callback returns truthy for any row. Short-circuits on first `true`.

## Function Signature
```
QuerySome(query, callback [, parallel, maxThreads, virtual])
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
- Add `some` method on `BxQuery` in `crates/matchbox-vm/src/datasource/mod.rs`
- Register as `querySome` BIF in `crates/matchbox-vm/src/bifs/datasource.rs`
- Register in `crates/matchbox-vm/src/bifs/mod.rs` under the `bif-datasource` feature block
- Gate behind `bif-datasource` feature flag
- Short-circuit: stop iterating on first `true` result
- Callback receives row struct, 1-based index, and query object

## Acceptance Criteria
- [ ] `query.some(callback)` returns `true` if any row matches
- [ ] `querySome(myQuery, callback)` works as standalone BIF
- [ ] Short-circuits on first `true`
- [ ] Returns `false` for empty query
- [ ] Callback receives `(rowStruct, rowIndex, queryObject)`
