# R010: Replace Eager Array Ranges With A Range Type

**Type:** Memory / CPU / Compatibility  
**Priority:** Medium  
**Related files:** `crates/matchbox-vm/src/vm/mod.rs`, `crates/matchbox-vm/src/vm/opcode.rs`
**Reference:** `~/dev/ortus-boxlang/BoxLang/src/main/java/ortus/boxlang/runtime/operators/Range.java`, `~/dev/ortus-boxlang/BoxLang/src/main/java/ortus/boxlang/runtime/types/Range.java`

## Problem

`op::RANGE` eagerly expands numeric ranges into arrays. This has poor memory behavior for large ranges and diverges from BoxLang JVM behavior, which creates a `Range` type.

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

## Acceptance Criteria

- [ ] Range creation does not allocate every element.
- [ ] Numeric ranges iterate in ascending and descending order.
- [ ] Exclusive bounds work.
- [ ] Large ranges can be created without high memory use.
- [ ] Behavior is aligned with BoxLang JVM range semantics where implemented.

