# C003: Add Range Operator Compilation

**Type:** AFK  
**Blocked by:** None

## What to build

Compile range operators (`..`, `>..`, `..<`, `>..<`) to create range objects usable in `for-in` loops.

### Approach
A range is a simple struct with fields: `start`, `end`, `inclusiveLeft`, `inclusiveRight`. The compiler emits bytecode to construct this struct at runtime.

### Compiler changes
For `a..b` (inclusive both ends):
1. Compile `a`, compile `b`
2. Emit CONSTANT("start") + CONSTANT(a) + CONSTANT("end") + CONSTANT(b) + CONSTANT(true) + CONSTANT(true)
3. Emit STRUCT(4) or equivalent struct construction

Or simpler: emit a `RANGE` opcode that takes `start`, `end`, two booleans and creates a range object.

### VM opcode
`RANGE` (76) — pops 4 values (start, end, incLeft, incRight), pushes range struct.

### for-in integration
The existing `for-in` loop iterates over arrays and structs. Ranges should be iterable. Either:
- Add range iteration to the ITER_NEXT opcode
- Or expand ranges into arrays at compile time (wasteful for large ranges)

### Test
```
var sum = 0;
for (var i in 1..5) {
    sum = sum + i;
}
println(sum); // expect 15 (1+2+3+4+5 = 15)

var result = "";
for (var i in 1..<4) {
    result = result & i;
}
println(result); // expect "123" (1,2,3)
```

## Acceptance criteria
- [ ] `a..b` creates inclusive range
- [ ] `a..<b` creates right-exclusive range
- [ ] `a>..b` creates left-exclusive range
- [ ] `a>..<b` creates both-exclusive range
- [ ] Ranges iterate correctly in for-in loops
- [ ] Integration test passes
