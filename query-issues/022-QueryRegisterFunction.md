# Implement `QueryRegisterFunction` BIF

**Status:** Not Started  
**Category:** Query Utility  
**Priority:** Low  

## BoxLang Reference
📁 `src/main/java/ortus/boxlang/runtime/bifs/global/query/QueryRegisterFunction.java`

## Description
Registers a custom scalar or aggregate function for use in Query of Queries (QoQ) SQL statements. Functions are cached for the lifetime of the runtime.

> **Note:** This is a global BIF only — NOT a member of the Query type in BoxLang.

## Function Signature
```
QueryRegisterFunction(name, function [, returnType, type])
```
| Argument     | Type     | Required | Default    | Description                         |
| ------------ | -------- | -------- | ---------- | ----------------------------------- |
| `name`       | String   | ✅        | —          | Name to register the function under |
| `function`   | Function | ✅        | —          | The UDF to register                 |
| `returnType` | String   | ❌        | `"Object"` | Return type of the function         |
| `type`       | String   | ❌        | `"scalar"` | `"scalar"` or `"aggregate"`         |

**Returns:** `void`

## Implementation Notes
- Register as `queryRegisterFunction` BIF in `crates/matchbox-vm/src/bifs/datasource.rs`
- Register in `crates/matchbox-vm/src/bifs/mod.rs` under the `bif-datasource` feature block
- Gate behind both `bif-datasource` and `qoq` feature flags
- Store registered functions in a global registry (similar to QoQ parse cache)
- Integrate with QoQ engine to resolve custom functions during query execution
- This is NOT a method on `BxQuery` — it's a standalone BIF

## Acceptance Criteria
- [ ] `queryRegisterFunction("myFunc", myUDF)` registers a function
- [ ] Registered functions can be used in QoQ SQL queries
- [ ] Scalar and aggregate function types are supported
- [ ] Functions persist for the runtime lifetime
