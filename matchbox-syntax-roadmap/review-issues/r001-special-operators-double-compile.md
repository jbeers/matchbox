# R001: Special Operators Double-Compile Operands

**Type:** Correctness / CPU / Stack hygiene  
**Priority:** High  
**Related files:** `crates/matchbox-compiler/src/compiler/mod.rs`, `crates/matchbox-vm/src/vm/mod.rs`

## Problem

Resolved in code: the special operators now lower before the eager binary fallback, so each operand is compiled exactly once.

`compile_expression` previously eagerly compiled both operands before dispatching on the operator. Several operator arms then compiled `left` and `right` again:

- `..`, `..<`, `>..`, `>..<`
- `contains`
- `instanceof`
- `castas`

This has three effects:

1. Operand side effects happen twice.
2. The first pair of operand values remains on the VM stack.
3. CPU work doubles for these operators.

Example risk:

```boxlang
var i = 0;
var r = (++i)..3;
// i should be 1 after range construction, not 2.
```

## Solution

Special-case operators that own operand compilation before the eager fallback path.

Recommended shape:

1. Handle short-circuit and special operators first.
2. Return immediately after emitting the operator bytecode.
3. Only use the common eager compile path for operators whose arm does not compile operands.

The unused `compile_range` helper can become the single implementation for range operators.

## Acceptance Criteria

- [x] `contains`, ranges, `instanceof`, and `castas` compile each operand exactly once.
- [x] Stack height is unchanged after expression statement cleanup.
- [x] Side-effect tests prove operands are evaluated once.
- [x] Range compilation uses one shared helper path.
- [x] No duplicated operand compilation remains in operator arms.

Note: `instanceof` and `castas` still have placeholder semantic behavior; that is tracked separately in R003.
