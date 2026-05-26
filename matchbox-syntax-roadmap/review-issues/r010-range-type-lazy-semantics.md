# R010: Replace Eager Array Ranges With A Range Type

**Type:** Memory / CPU / Compatibility  
**Priority:** Medium  
**Status:** Implemented
**Related files:** `crates/matchbox-vm/src/vm/mod.rs`, `crates/matchbox-vm/src/vm/opcode.rs`
**Reference:** `~/dev/ortus-boxlang/BoxLang/src/main/java/ortus/boxlang/runtime/operators/Range.java`, `~/dev/ortus-boxlang/BoxLang/src/main/java/ortus/boxlang/runtime/types/Range.java`

## Problem

`op::RANGE` now lowers to a lazy `Range` GC object instead of eagerly expanding numeric ranges into arrays. This removes the large-array allocation behavior and aligns the VM with BoxLang JVM range semantics for the implemented cases.

Current limitations:

- Large ranges allocate all values immediately.
- Open-ended ranges are impossible.
- Non-numeric ranges are rejected.
- Range methods and typed behavior cannot be represented.

## Solution

Add a VM `Range` GC object or native object.

Suggested shape:

1. Store start, end, exclusivity flags, direction, and optional step.
2. Implement iteration over ranges without materializing all values.
3. Teach `len`, `for-in`, indexing if applicable, and `contains` about the range type.
4. Preserve eager array conversion only where a BIF explicitly asks for an array.

## Test

```boxlang
var r = 1..1000000;
var count = 0;
for (var n in r) {
    count++;
    if (count == 3) {
        break;
    }
}

if (count != 3) {
    throw "range iteration failed";
}
```

## Notes

Implemented on this branch:

- `op::RANGE` now allocates a `GcObject::Range` instead of expanding an array.
- Range iteration is lazy through `ITER_NEXT`.
- `len` and `contains` understand the range type.
- Ascending and descending iteration, exclusive bounds, and large-range early exit are covered by `tests/scripts/vm_ranges.bxs`.

## Acceptance Criteria

- [x] Range creation does not allocate every element.
- [x] Numeric ranges iterate in ascending and descending order.
- [x] Exclusive bounds work.
- [x] Large ranges can be created without high memory use.
- [x] Behavior is aligned with BoxLang JVM range semantics where implemented.
