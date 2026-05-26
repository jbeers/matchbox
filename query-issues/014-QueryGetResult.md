# Implement `QueryGetResult` BIF

**Status:** Not Started  
**Category:** Query Access  
**Priority:** Low  

## BoxLang Reference
📁 `src/main/java/ortus/boxlang/runtime/bifs/global/query/QueryGetResult.java`

## Description
Returns the result metadata of a query execution. This typically includes information like the SQL statement executed, execution time, record count, and other query execution metadata.

## Function Signature
```
QueryGetResult(query)
```
| Argument | Type  | Required | Description      |
| -------- | ----- | -------- | ---------------- |
| `query`  | Query | ✅        | The query object |

**Returns:** `Struct` — metadata about the query result

## Implementation Notes
- Add `getResult` method on `BxQuery` in `crates/matchbox-vm/src/datasource/mod.rs`
- Register as `queryGetResult` BIF in `crates/matchbox-vm/src/bifs/datasource.rs`
- Register in `crates/matchbox-vm/src/bifs/mod.rs` under the `bif-datasource` feature block
- Gate behind `bif-datasource` feature flag
- Return a struct with at minimum: `recordCount`, `columnList`, `sql` (if available)
- May also include: `executionTime`, `cached`

## Acceptance Criteria
- [ ] `query.getResult()` returns a struct with metadata
- [ ] `queryGetResult(myQuery)` works as standalone BIF
- [ ] Struct contains `recordCount` and `columnList` at minimum
- [ ] `sql` field populated if query came from `queryExecute()`
