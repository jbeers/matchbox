# Implement `QueryMap` BIF

**Status:** Not Started  
**Category:** Query Transform  
**Priority:** High  

## BoxLang Reference
📁 `src/main/java/ortus/boxlang/runtime/bifs/global/query/QueryMap.java`

## Description
Iterates over each row in a query and invokes a callback function that returns a transformed value for each row. Returns a new query with the transformed data.

## Function Signature
```
QueryMap(query, callback [, parallel, maxThreads, virtual])
```
| Argument     | Type     | Required | Default | Description                                        |
| ------------ | -------- | -------- | ------- | -------------------------------------------------- |
| `query`      | Query    | ✅        | —       | The query to transform                             |
| `callback`   | Function | ✅        | —       | Transformer receiving (row, index, query) → struct |
| `parallel`   | Boolean  | ❌        | `false` | Execute in parallel                                |
| `maxThreads` | Integer  | ❌        | —       | Max threads for parallel mode                      |
| `virtual`    | Boolean  | ❌        | `false` | Use virtual threads                                |

**Returns:** `Query` — new query with transformed data

## Implementation Notes
- Add `map` method on `BxQuery` in `crates/matchbox-vm/src/datasource/mod.rs`
- Register as `queryMap` BIF in `crates/matchbox-vm/src/bifs/datasource.rs`
- Register in `crates/matchbox-vm/src/bifs/mod.rs` under the `bif-datasource` feature block
- Gate behind `bif-datasource` feature flag
- Returns a **new** query object
- The callback should return a struct with column name/value pairs
- Column structure of new query is derived from callback return values

## Acceptance Criteria
- [ ] `query.map(callback)` returns new transformed query
- [ ] `queryMap(myQuery, callback)` works as standalone BIF
- [ ] Original query is not modified
- [ ] Callback receives `(rowStruct, rowIndex, queryObject)`
- [ ] Callback return value populates the new row
