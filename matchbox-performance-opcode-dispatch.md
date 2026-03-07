# Optimization Plan: Optimized OpCode Dispatch

## Objective
Reduce the CPU overhead of the VM's main execution loop. Currently, the `match` statement inside a `loop` creates a central bottleneck and can lead to branch mispredictions in the CPU's pipeline.

## Proposed Solution
Move from a standard `match` loop to a more efficient dispatch mechanism, often referred to as "Computed GOTOs" or "Tail-Call Dispatch."

### Implementation Details
1.  **Dispatch Table**: Ensure the Rust compiler generates a jump table for the `match` statement (the current `OpCode` enum is already well-ordered for this).
2.  **Instruction Pre-fetching**: Fetch the next opcode at the end of the current opcode's execution block to allow the CPU to overlap the fetch and execute stages.
3.  **Tail-Call Optimization**: Explore using the `#[musttail]` attribute (if available on the target toolchain) to transform opcode handlers into separate functions that "jump" to each other, eliminating the stack frame overhead of the central loop.
4.  **Hot-Path Ordering**: Reorder the `match` arms in `run_fiber` so that the most frequent opcodes (`OpGetLocal`, `OpSetLocal`, `OpAdd`, `OpJumpIfFalse`) are at the top, allowing the compiler to generate more efficient branch logic.

## Success Criteria
- Reduction in "cycles per instruction" (CPI) as measured by a profiler.
- Higher throughput in branch-heavy code (complex logic and nested loops).
