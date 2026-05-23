# R007: Fix Destructuring Evaluation Semantics

**Type:** Correctness / CPU  
**Priority:** Medium  
**Related files:** `crates/matchbox-compiler/src/parser/mod.rs`, `crates/matchbox-compiler/src/compiler/mod.rs`
**Status:** Resolved in code

## Problem

Destructuring compiles the source expression once per binding. This causes duplicated side effects and unnecessary CPU work.

It also treats all destructuring as object member access. Array destructuring syntax is accepted by lookahead but `[a] = arr` would be lowered like `arr.a`, not `arr[1]`.

Example risk:

```boxlang
function makeObj() {
    println("called");
    return { a: 1, b: 2 };
}

{ a, b } = makeObj();
// "called" should be printed once, not twice.
```

## Solution

Compile the source expression once, store it in a temporary local, then read each binding from that temporary.

Implementation outline:

1. Extend `StatementKind::Destructure` to distinguish object and array patterns.
2. Add compiler support for a hidden temporary local.
3. Object destructuring reads named members.
4. Array destructuring reads 1-based indexes.
5. Preserve rename/default/rest semantics if parsed.

## Acceptance Criteria

- [x] Destructuring source expressions evaluate exactly once.
- [x] Object destructuring reads members by key.
- [x] Array destructuring reads 1-based array indexes.
- [x] Renamed object bindings work.
- [x] Tests cover side-effecting source expressions.
