# N011: Utility BIFs

**Type:** Runtime
**Priority:** Medium

## What to build

Implement the common utility and type-checking helpers used across scripts and
templates.

## Acceptance criteria

- [x] `writeOutput()` writes to the output buffer
- [x] UUID and sleep helpers work
- [x] Type checks return the expected booleans
- [x] `duplicate()` deep-copies structured values

## Completed

Implemented the utility helper surface in the VM/runtime:

- `writeOutput()` writes to the buffered template output stream
- `createUUID()` and `createGUID()` return UUID-formatted values
- `sleep()` and `yield()` are available in runtime scripts
- `isArray()`, `isStruct()`, `isBoolean()`, `isString()`, `isDate()`, and `isObject()` return the expected type predicates
- `duplicate()` deep-copies nested arrays and structs and is available as a member call on arrays, structs, and datetimes
