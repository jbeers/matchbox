# Optimization Plan: Full BoxString Integration

## Objective
Reduce heap pressure and "pointer chasing" by integrating the custom `BoxString` (Small String Optimization) throughout the entire VM and Compiler.

## Proposed Solution
Replace all occurrences of standard Rust `String` with the optimized `BoxString` in the core type definitions.

### Implementation Details
1.  **Type Refactor**:
    *   Update `crates/matchbox-vm/src/types/mod.rs`: Change `Constant::String(String)` to `Constant::String(BoxString)`.
    *   Update `crates/matchbox-vm/src/vm/gc.rs`: Change `GcObject::String(String)` to `GcObject::String(BoxString)`.
2.  **Zero-Allocation Identifiers**:
    *   Variable names and property keys under 22 bytes will now be stored **inline** within the constant pool and the heap.
3.  **Rope-Based Concatenation**:
    *   Update `OpAdd` and `OpStringConcat` to use `BoxString::concat`, which enables $O(1)$ string building for large documents or strings.
4.  **Lazy Flattening**:
    *   Ensure that `BoxString::flatten()` is only called when strictly necessary (e.g., passing a string to a regex engine or JNI).

## Success Criteria
- VM memory usage reduced for applications with many small strings (identifiers).
- String concatenation performance improves from $O(N)$ to $O(1)$.
