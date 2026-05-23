# 017: Add Remaining Operators (`XOR`, `EQV`, `^`, `&=`, `&`)

**Type:** AFK  
**Blocked by:** #003 (Wire New Parser)

## What to build

Add the remaining BoxLang operators that are not covered by other issues.

BoxLang operators:
```
a XOR b      — logical exclusive OR
a EQV b      — logical equivalence (XNOR)
a ^ b        — power (exponentiation)
a &= b       — compound string concatenation assignment
a & b        — string concatenation (already supported, verify not confused with bitwise b&)
```

## Delivery

- **Parser:** Add to Pratt parser:
  - `XOR`: precedence between `&&` and `||`
  - `EQV`: same precedence as `XOR`
  - `^` (power): highest binary precedence, right-associative
  - `&=` (compound concat assign): handled like `+=`/`-=` — desugars to `a = a & b`
  - `&` (string concat): already supported, verify correct precedence vs bitwise `b&`
- **Compiler:** Emit bytecode for each operation:
  - `XOR`/`EQV`: emit logical operation bytecode
  - `^`: emit power bytecode (or call math.pow runtime function)
  - `&=`: desugared as compound assignment, left to compiler to handle
- **Test:** Integration test for each operator.

## Acceptance criteria

- [ ] `true XOR false` → `true`; `true XOR true` → `false`
- [ ] `true EQV true` → `true`; `true EQV false` → `false`
- [ ] `2 ^ 3` → `8`; `2 ^ 0` → `1`
- [ ] `a ^ b ^ c` is right-associative: `a ^ (b ^ c)`
- [ ] `a &= "suffix"` desugars to `a = a & "suffix"`
- [ ] `"hello" & " " & "world"` → `"hello world"` (already works, regress-test)
- [ ] `a & b` is string concat, `a b& b` is bitwise AND — no confusion
- [ ] Integration test passes
