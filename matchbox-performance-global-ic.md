# Optimization Plan: Global Inline Caching (IC)

## Objective
Eliminate the overhead of `HashMap` lookups for global variables in the VM's hot path. In BoxLang, global variables (like `i` in a loop) are accessed millions of times. Currently, each access triggers a string hash and a map traversal.

## Proposed Solution
Leverage the existing `Chunk::caches` infrastructure to implement a Monomorphic Inline Cache for `OpGetGlobal` and `OpSetGlobal`.

### Implementation Details
1.  **Cache Structure**: Add a new variant to `IcEntry` or reuse the existing `Monomorphic` variant to store a "Global Slot" identifier or a stable pointer to the value.
2.  **Stable Storage**: Modify the `VM` to store globals in a `Vec<BxValue>` (the "Global Table") and use a `HashMap<String, usize>` to map names to indices. 
3.  **The Fast Path**:
    *   When `OpGetGlobal(idx)` is executed, the VM checks the `IcEntry` at the current instruction pointer.
    *   If a hit occurs, it directly accesses the `Global Table` using the cached index—a simple array lookup.
    *   If a miss occurs, it performs the full `HashMap` lookup, populates the cache with the resulting index, and proceeds.
4.  **Compiler Support**: No changes needed to the compiler, as it already provides the identifier index via the opcode.

## Success Criteria
- The 10-million iteration loop should see a ~30-50% reduction in execution time as string hashing is eliminated from the loop body.
