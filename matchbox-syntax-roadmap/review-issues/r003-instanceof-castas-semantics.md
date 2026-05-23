# R003: Implement Real `instanceof` And `castas` Semantics

**Type:** Correctness / Compatibility  
**Priority:** High  
**Related files:** `crates/matchbox-compiler/src/compiler/mod.rs`, `crates/matchbox-vm/src/vm/mod.rs`
**Reference:** `~/dev/ortus-boxlang/BoxLang/src/main/java/ortus/boxlang/compiler/javaboxpiler/transformer/expression/BoxBinaryOperationTransformer.java`

## Status

Resolved in code. The compiler now lowers `instanceof` and `castas` to real VM opcodes, and the VM performs actual type checks / casts instead of returning placeholder values.

## Problem

`instanceof` and `castas` used to be semantic stubs. They now evaluate both operands once, dispatch to the VM, and apply real runtime semantics.

## Solution

Add real runtime operations for these operators.

Suggested pieces:

1. Add `INSTANCEOF` and `CASTAS` opcodes, or dispatch through equivalent native operators.
2. Resolve type operands from string literals, identifiers, class values, interface values, and built-in type names.
3. Implement primitive checks for `string`, `numeric`, `integer`, `boolean`, `array`, `struct`, `function`, `class`, and `null`.
4. Implement instance/class/interface checks using VM class metadata.
5. Make `castas` return the cast value or throw on invalid conversion.

## Test

```boxlang
if (!("x" instanceof "string")) {
    throw "string instanceof failed";
}

if (123 instanceof "string") {
    throw "numeric should not be a string";
}

var x = "123" castas "numeric";
if (x + 1 != 124) {
    throw "castas numeric failed";
}
```

## Acceptance Criteria

- [x] `instanceof` returns false for mismatched built-in types.
- [x] `instanceof` supports class and interface checks where metadata exists.
- [x] `castas` performs real conversions for core simple types.
- [x] Invalid `castas` raises an error.
- [x] Operators evaluate operands exactly once.
