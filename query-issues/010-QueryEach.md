# Implement `QueryEach` BIF

**Status:** Not Started  
**Category:** Query Iteration  
**Priority:** High  

## BoxLang Reference
📁 `src/main/java/ortus/boxlang/runtime/bifs/global/query/QueryEach.java`

## Description
Iterates over each row in a query and invokes a callback function for side effects. Supports parallel execution with optional thread configuration.

## Function Signature
```
QueryEach(query, callback [, parallel, maxThreads, ordered, virtual])
```
| Argument     | Type     | Required | Default | Description                            |
| ------------ | -------- | -------- | ------- | -------------------------------------- |
| `query`      | Query    | ✅        | —       | The query to iterate                   |
| `callback`   | Function | ✅        | —       | Callback receiving (row, index, query) |
| `parallel`   | Boolean  | ❌        | `false` | Execute in parallel                    |
| `maxThreads` | Integer  | ❌        | —       | Max threads for parallel mode          |
| `ordered`    | Boolean  | ❌        | `false` | Preserve order in parallel mode        |
| `virtual`    | Boolean  | ❌        | `false` | Use virtual threads                    |

**Returns:** `void` (null)

## Implementation Notes
- Add `each` method on `BxQuery` in `crates/matchbox-vm/src/datasource/mod.rs`
- Register as `queryEach` BIF in `crates/matchbox-vm/src/bifs/datasource.rs`
- Register in `crates/matchbox-vm/src/bifs/mod.rs` under the `bif-datasource` feature block
- Gate behind `bif-datasource` feature flag
- The callback receives the row as a struct, the current row index (1-based), and the query object
- Parallel execution may be deferred or implemented as serial-only initially
- Update `currentRow` cursor during iteration

## Acceptance Criteria
- [ ] `query.each(callback)` iterates all rows and invokes callback
- [ ] `queryEach(myQuery, callback)` works as standalone BIF
- [ ] Callback receives `(rowStruct, rowIndex, queryObject)`
- [ ] Serial execution is fully functional
- [ ] Parallel execution (optional, can be stubbed or deferred)
