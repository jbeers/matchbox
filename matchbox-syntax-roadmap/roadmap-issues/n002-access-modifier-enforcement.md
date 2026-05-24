# N002: Access Modifier Enforcement

**Type:** Compiler / Runtime validation
**Priority:** High

## What to build

Validate abstract/final/static access modifier semantics at compile time so the
parser metadata already being preserved becomes enforceable.

## Status

Completed. The compiler now rejects abstract instantiation, final inheritance
and method overrides, and invalid static `this` / `super` references.

## Acceptance criteria

- [x] Instantiating an abstract class fails
- [x] Extending a final class fails
- [x] Overriding a final method fails
- [x] `this` and `super` are rejected in static context where applicable
- [x] Integration tests cover the invalid cases
