# Implement `QueryRecordCount` BIF

**Status:** Not Started  
**Category:** Query Introspection  
**Priority:** Medium  

## BoxLang Reference
📁 `src/main/java/ortus/boxlang/runtime/bifs/global/query/QueryRecordCount.java`

## Description
Returns the number of records (rows) in a query object. 

> **Note:** Matchbox already has `recordCount` as a property on `BxQuery`, but does NOT expose it as a standalone BIF `queryRecordCount()`.

## Function Signature
```
QueryRecordCount(query)
```
| Argument | Type  | Required | Description      |
| -------- | ----- | -------- | ---------------- |
| `query`  | Query | ✅        | The query object |

**Returns:** `Integer` — number of rows

## Implementation Notes
- Register as `queryRecordCount` BIF in `crates/matchbox-vm/src/bifs/datasource.rs`
- Register in `crates/matchbox-vm/src/bifs/mod.rs` under the `bif-datasource` feature block
- Gate behind `bif-datasource` feature flag
- Simply delegates to reading the `recordCount` property of `BxQuery`

## Acceptance Criteria
- [ ] `queryRecordCount(myQuery)` returns the row count
- [ ] Same result as `myQuery.recordCount`
