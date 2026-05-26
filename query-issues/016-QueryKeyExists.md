# Implement `QueryKeyExists` BIF

**Status:** Not Started  
**Category:** Query Introspection  
**Priority:** Medium  

## BoxLang Reference
📁 `src/main/java/ortus/boxlang/runtime/bifs/global/query/QueryKeyExists.java`

## Description
Checks whether a given key (column name) exists in the query object. Case-insensitive. Alias for `QueryColumnExists`.

## Function Signature
```
QueryKeyExists(query, key)
```
| Argument | Type   | Required | Description              |
| -------- | ------ | -------- | ------------------------ |
| `query`  | Query  | ✅        | The query object         |
| `key`    | String | ✅        | The column name to check |

**Returns:** `Boolean`

## Implementation Notes
- Add `keyExists` method on `BxQuery` in `crates/matchbox-vm/src/datasource/mod.rs` (can delegate to `columnExists`)
- Register as `queryKeyExists` BIF in `crates/matchbox-vm/src/bifs/datasource.rs`
- Register in `crates/matchbox-vm/src/bifs/mod.rs` under the `bif-datasource` feature block
- Gate behind `bif-datasource` feature flag
- Essentially identical to `QueryColumnExists` — can share implementation
- Case-insensitive matching

## Acceptance Criteria
- [ ] `query.keyExists("myCol")` returns `true`/`false`
- [ ] `queryKeyExists(myQuery, "myCol")` works as standalone BIF
- [ ] Case-insensitive matching
- [ ] Returns `false` for non-existent column
