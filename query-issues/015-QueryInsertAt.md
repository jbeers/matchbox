# Implement `QueryInsertAt` BIF

**Status:** Not Started  
**Category:** Query Mutation  
**Priority:** Medium  

## BoxLang Reference
📁 `src/main/java/ortus/boxlang/runtime/bifs/global/query/QueryInsertAt.java`

## Description
Inserts rows from one query into another at a specific position (1-based).

## Function Signature
```
QueryInsertAt(query, value, position)
```
| Argument   | Type    | Required | Description                                |
| ---------- | ------- | -------- | ------------------------------------------ |
| `query`    | Query   | ✅        | The destination query                      |
| `value`    | Query   | ✅        | The source query containing rows to insert |
| `position` | Integer | ✅        | 1-based position to insert at              |

**Returns:** `Query` (the modified query)

## Implementation Notes
- Add `insertAt` method on `BxQuery` in `crates/matchbox-vm/src/datasource/mod.rs`
- Register as `queryInsertAt` BIF in `crates/matchbox-vm/src/bifs/datasource.rs`
- Register in `crates/matchbox-vm/src/bifs/mod.rs` under the `bif-datasource` feature block
- Gate behind `bif-datasource` feature flag
- Insert rows at the given position; shift existing rows down
- Only matching columns are inserted; extra columns in source are ignored
- Missing columns in source get NULL values

## Acceptance Criteria
- [ ] `query.insertAt(sourceQuery, 3)` inserts rows at position 3
- [ ] `queryInsertAt(dest, src, 3)` works as standalone BIF
- [ ] Existing rows at/after position are shifted down
- [ ] Column matching works correctly
- [ ] Returns the modified query
