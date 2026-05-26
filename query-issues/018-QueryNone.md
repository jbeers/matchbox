# Implement `QueryNone` BIF

**Status:** Not Started  
**Category:** Query Predicate  
**Priority:** Medium  

## BoxLang Reference
📁 `src/main/java/ortus/boxlang/runtime/bifs/global/query/QueryNone.java`

## Description
Tests whether **no** rows in a query satisfy the predicate callback function. Returns `true` only if the callback returns falsy for all rows. Short-circuits on first `true`. In BoxLang, `QueryNone` extends `QuerySome` and negates its result.

## Function Signature
```
QueryNone(query, callback [, parallel, maxThreads, virtual])
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
- Add `none` method on `BxQuery` in `crates/matchbox-vm/src/datasource/mod.rs`
- Register as `queryNone` BIF in `crates/matchbox-vm/src/bifs/datasource.rs`
- Register in `crates/matchbox-vm/src/bifs/mod.rs` under the `bif-datasource` feature block
- Gate behind `bif-datasource` feature flag
- Can be implemented as `!query.some(callback)` — negate the result of `some`
- Short-circuit on first truthy callback result

## Acceptance Criteria
- [ ] `query.none(callback)` returns `true` if no rows match
- [ ] `queryNone(myQuery, callback)` works as standalone BIF
- [ ] Short-circuits on first match
- [ ] Returns `true` for empty query
- [ ] Equivalent to `!query.some(callback)`
