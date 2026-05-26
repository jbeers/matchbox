# R002: `rethrow` Must Preserve The Active Exception

**Type:** Correctness / Compatibility  
**Priority:** High  
**Related files:** `crates/matchbox-compiler/src/compiler/mod.rs`, `crates/matchbox-vm/src/vm/mod.rs`
**Reference:** `~/dev/ortus-boxlang/BoxLang/src/main/java/ortus/boxlang/compiler/javaboxpiler/transformer/statement/BoxRethrowTransformer.java`

## Problem

Resolved in code: `StatementKind::Rethrow` now lowers to the current catch variable and rethrows the active exception value instead of synthesizing a new string.

The JVM reference lowers `rethrow` to the current context's rethrow behavior, which preserves and rethrows the active exception.

## Solution

Track the active exception in the catch-handling lowering and rethrow that local value. Error if `rethrow` executes outside a catch context.

## Test

```boxlang
var originalMessage = "";
try {
    try {
        throw(message="inner", type="Custom.Type");
    } catch (e) {
        rethrow;
    }
} catch (e) {
    originalMessage = e.message;
}

if (originalMessage != "inner") {
    throw "rethrow did not preserve exception";
}
```

## Acceptance Criteria

- [x] `rethrow` inside `catch` rethrows the original exception value.
- [x] Structured exception fields are preserved.
- [x] Stack trace is not replaced by `"Rethrown exception"`.
- [x] `rethrow` outside `catch` reports a runtime or compile error.
- [x] Integration test covers nested catch/rethrow behavior.
