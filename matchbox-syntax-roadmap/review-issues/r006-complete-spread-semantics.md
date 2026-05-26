# R006: Complete Spread Semantics

**Type:** Correctness / Compatibility  
**Priority:** Medium  
**Related issue:** `c008-spread-desugaring.md`
**Related files:** `crates/matchbox-compiler/src/parser/mod.rs`, `crates/matchbox-compiler/src/compiler/mod.rs`, `crates/matchbox-vm/src/vm/mod.rs`
**Status:** Resolved in code

## Problem

Array literal spread works, but the remaining spread forms are incomplete:

- Function-call spread is parsed but compiled as a single argument value.
- Struct literal spread is not parsed.
- Non-array values in array spread are silently appended rather than rejected or handled according to BoxLang semantics.

This does not meet the existing C008 acceptance criteria.

## Solution

Complete spread lowering across all accepted positions:

1. Function call spread: expand array argument values and compute the final argument count.
2. Struct spread: merge keys from spread structs in source order.
3. Decide and document runtime behavior for spreading invalid values.
4. Add opcodes or compiler desugaring that avoids excessive intermediate arrays where possible.

## Test

```boxlang
function sum(a, b, c) {
    return a + b + c;
}

var args = [1, 2];
if (sum(...args, 3) != 6) {
    throw "function spread failed";
}

var base = { b: 2, c: 3 };
var merged = { a: 1, ...base, d: 4 };
if (merged.a != 1 || merged.b != 2 || merged.c != 3 || merged.d != 4) {
    throw "struct spread failed";
}
```

## Acceptance Criteria

- [x] Spread in function arguments works.
- [x] Mixed spread and normal function arguments work.
- [x] Spread in struct literals works.
- [x] Invalid spread values have defined behavior and tests.
- [x] Existing array spread tests continue to pass.
