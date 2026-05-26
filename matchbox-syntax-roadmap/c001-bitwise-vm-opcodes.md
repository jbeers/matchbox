# C001: Add Bitwise Operator VM Opcodes

**Type:** AFK  
**Blocked by:** None — can start immediately

## What to build

Add VM opcodes and compiler support for all 7 bitwise operators: `b|`, `b&`, `b^`, `b~`, `b<<`, `b>>`, `b>>>`.

### VM opcodes to add
- `BIT_OR` (66) — pop two values, push `a | b` (integer bitwise OR)
- `BIT_AND` (67) — pop two values, push `a & b`
- `BIT_XOR` (68) — pop two values, push `a ^ b`
- `BIT_NOT` (69) — pop one value, push `~a` (bitwise complement)
- `BIT_SHL` (70) — pop two values, push `a << b`
- `BIT_SHR` (71) — pop two values, push `a >> b` (signed)
- `BIT_USHR` (72) — pop two values, push `a >>> b` (unsigned)

### Compiler changes
Map tokenized operators to the correct opcodes in `compile_expression`:
- `"b|"` → BIT_OR
- `"b&"` → BIT_AND
- `"b^"` → BIT_XOR
- `"b~"` → BIT_NOT (unary prefix)
- `"b<<"` → BIT_SHL
- `"b>>"` → BIT_SHR
- `"b>>>"` → BIT_USHR

### VM interpreter
Add opcode cases in the VM main loop. For NaN-boxed values, bitwise ops only apply to numeric (float→int→op→float) or integer values. Strings/objects should throw.

### Test
Integration test `.bxs` file exercising each operator with expected results:
```
println(5 b| 3);   // expect 7
println(5 b& 3);   // expect 1
println(5 b^ 3);   // expect 6
println(b~0);      // expect -1
println(8 b<< 1);  // expect 16
println(8 b>> 1);  // expect 4
```

## Acceptance criteria
- [ ] All 7 bitwise opcodes added to opcode.rs
- [ ] Compiler emits correct opcode for each bitwise operator
- [ ] VM interpreter handles each opcode correctly
- [ ] Non-numeric values throw ExpressionException
- [ ] Integration test passes with expected outputs
