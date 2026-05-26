# N003: DateTime Core And Date BIFs

**Type:** Runtime
**Priority:** High

## What to build

Add the core `DateTime` runtime type plus the date construction, arithmetic,
comparison, and formatting BIFs the language expects.

## Status

Completed. `DateTime` is a first-class runtime value with construction,
arithmetic, comparison, formatting, parsing, and member-function support.

## Acceptance criteria

- [x] `now()` returns a current `DateTime`
- [x] `createDate()` and `createDateTime()` work
- [x] `dateAdd()` and `dateDiff()` work
- [x] `dateFormat()` and `parseDateTime()` work
- [x] Date comparison operators behave correctly
