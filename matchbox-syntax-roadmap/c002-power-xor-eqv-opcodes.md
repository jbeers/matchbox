# C002: Add Power, XOR, EQV Operator Opcodes

**Type:** AFK  
**Blocked by:** None

## What to build

Add VM opcodes and compiler support for power (`^`), XOR, and EQV operators.

### VM opcodes
- `POW` (73) — pop two values, push `a ** b` (exponentiation)
- `XOR_OP` (74) — pop two values, push `a XOR b` (logical exclusive OR: true if exactly one is truthy)
- `EQV_OP` (75) — pop two values, push `a EQV b` (logical equivalence: true if both have same truthiness)

### Compiler changes
- `"^"` → POW (note: right-associative, handled by Pratt parser)
- `"xor"` → XOR_OP
- `"eqv"` → EQV_OP

### Test
```
println(2 ^ 3);     // expect 8
println(true XOR false);  // expect true
println(true XOR true);   // expect false
println(true EQV true);   // expect true
println(true EQV false);  // expect false
```

## Acceptance criteria
- [ ] POW opcode returns correct exponentiation
- [ ] Power is right-associative: `2 ^ 3 ^ 2` = `2 ^ 9` = 512
- [ ] XOR returns true when exactly one operand is truthy
- [ ] EQV returns true when both operands have same truthiness
- [ ] Integration test passes
