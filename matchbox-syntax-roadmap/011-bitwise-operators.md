# 011: Add Bitwise Operators

**Type:** AFK  
**Blocked by:** #003 (Wire New Parser)

## What to build

Add all BoxLang bitwise operators through all compiler layers.

BoxLang operators:
```
b|   — bitwise OR
b&   — bitwise AND
b^   — bitwise XOR
b~   — bitwise complement (unary prefix)
b<<  — bitwise signed left shift
b>>  — bitwise signed right shift
b>>> — bitwise unsigned right shift
```

Note: plain `&` is string concatenation in BoxLang, not bitwise AND. The `b` prefix distinguishes bitwise.

## Delivery

- **Lexer:** Tokenize `b|`, `b&`, `b^`, `b~`, `b<<`, `b>>`, `b>>>` as distinct tokens
- **Parser:** Add these operators to the Pratt parser with appropriate precedence. Bitwise complement (`b~`) is a unary prefix operator. The rest are binary. Precedence (high to low): `b~` (unary), shifts, `b&`, `b^`, `b|` (lowest).
- **AST:** Existing `Binary` and `UnaryNot`/new unary variant can represent these. Alternatively add dedicated expression kinds for bitwise if the compiler needs to distinguish them.
- **Compiler:** Emit bytecode for each bitwise operation. If the VM doesn't have bitwise opcodes, implement them as built-in function calls or add VM opcodes.
- **Test:** Integration test exercising each bitwise operator with expected results.

## Acceptance criteria

- [ ] `a b| b` tokenizes and parses correctly
- [ ] `a b& b` tokenizes and parses correctly
- [ ] `a b^ b` tokenizes and parses correctly
- [ ] `b~a` tokenizes and parses as unary prefix
- [ ] `a b<< b` tokenizes and parses correctly
- [ ] `a b>> b` tokenizes and parses correctly
- [ ] `a b>>> b` tokenizes and parses correctly
- [ ] `&` still works as string concatenation (not confused with `b&`)
- [ ] All bitwise operations produce correct numeric results at runtime
- [ ] Integration test passes
