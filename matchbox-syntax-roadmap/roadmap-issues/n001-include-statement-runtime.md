# N001: Include Statement Runtime

**Type:** Runtime / Compiler
**Priority:** High

## What to build

`include` should load and execute another BoxLang file with the same runtime
scope rules as the caller.

## Status

Completed. Static string includes are resolved by the compiler and inlined into
the current compile unit. Runtime fallback remains available for non-literal
paths.

## Acceptance criteria

- [x] Static string includes resolve to a file path and execute successfully
- [x] Variables defined in the included file remain visible to the caller
- [x] Missing include files produce a compile/runtime error with location
- [x] Integration coverage proves nested include behavior
