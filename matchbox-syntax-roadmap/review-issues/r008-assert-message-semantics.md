# R008: Preserve `assert` Message Semantics

Status: Implemented

**Type:** Correctness / Compatibility  
**Priority:** Medium  
**Related files:** `crates/matchbox-compiler/src/parser/mod.rs`, `crates/matchbox-compiler/src/compiler/mod.rs`
**Reference:** `~/dev/ortus-boxlang/BoxLang/src/main/java/ortus/boxlang/compiler/javaboxpiler/transformer/statement/BoxAssertTransformer.java`

## Problem

The parser accepts `assert condition : message`, but the compiler ignores the message and always throws `"Assertion failed"`.

The JVM reference passes the optional message to `Assert.invoke`.

## Solution

Compile the optional message expression into the thrown error.

Possible implementation:

1. If no message is supplied, preserve the current default.
2. If a message is supplied, evaluate it only when the assertion fails.
3. Throw an exception object or message matching the VM's standard error shape.

## Test

```boxlang
try {
    assert false : "custom failure";
} catch (e) {
    if (!e.message contains "custom failure") {
        throw "assert message was lost";
    }
}
```

## Acceptance Criteria

- [x] Message expression is evaluated only on assertion failure.
- [x] Custom message appears in the thrown exception.
- [x] Default message still works when no message is supplied.
- [x] Integration tests cover both forms.
