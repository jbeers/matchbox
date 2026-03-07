# Optimization Plan: Primitive-Specific Opcodes

## Objective
Bypass dynamic type-checking and NaN-unboxing logic for operations where the types are known or highly predictable.

## Proposed Solution
Introduce specialized opcodes for 32-bit integer math and optimized loop counters.

### Implementation Details
1.  **New Opcodes**:
    *   `OpAddInt`, `OpSubInt`: Performs raw `i32` math without checking for floats or strings.
    *   `OpLtInt`, `OpGtInt`: Faster comparison for integers.
2.  **Specialized Loop Opcode**:
    *   `OpForLoopStep`: A combined opcode that increments a local variable, checks a limit, and jumps back—all in a single VM instruction. This eliminates 3 separate opcodes per loop iteration.
3.  **Compiler Specialization**:
    *   Update the compiler to detect numeric literals in `for` loops. If a loop is written as `for (i=0; i < 10; i++)`, the compiler can emit the `Int` variants of the opcodes.
4.  **Speculative Execution**:
    *   Optionally, the VM can "speculate" that a variable is an integer. If the speculation fails (e.g., someone assigns a string to the loop counter), it falls back to the generic `OpAdd`.

## Success Criteria
- Near-native performance for tight integer-based loops.
- Significant reduction in the number of instructions executed for standard `for` loops.
