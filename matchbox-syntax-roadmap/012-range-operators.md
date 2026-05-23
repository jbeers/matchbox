# 012: Add Range Operators

**Type:** AFK  
**Blocked by:** #003 (Wire New Parser)

## What to build

Add BoxLang range operators through all compiler layers.

BoxLang operators:
```
a..b   — inclusive range [a, b]
a>..b  — left-exclusive range (a, b]
a..<b  — right-exclusive range [a, b)
a>..<b — both-exclusive range (a, b)
```

Range operators create range objects that are typically used in `for-in` loops or array slicing. The runtime representation can be a simple struct with `start`, `end`, `inclusiveLeft`, `inclusiveRight` fields.

## Delivery

- **Lexer:** Tokenize `..`, `>..`, `..<`, `>..<` as distinct tokens
- **Parser:** Add range operators to Pratt parser. All range operators share the same precedence level (below comparison, above bitwise OR). Ranges are left-associative.
- **AST:** Add `ExpressionKind::Range { left, operator, right }` where operator distinguishes inclusive/exclusive variants.
- **Compiler:** Emit bytecode that constructs a range object at runtime. This could be a `BxStruct` with start/end/exclusivity flags, or a dedicated range value type.
- **Test:** Integration test creating ranges and iterating over them in `for-in` loops.

## Acceptance criteria

- [ ] `a..b` tokenizes and parses as inclusive range
- [ ] `a>..b` tokenizes and parses as left-exclusive range
- [ ] `a..<b` tokenizes and parses as right-exclusive range
- [ ] `a>..<b` tokenizes and parses as both-exclusive range
- [ ] Ranges work in `for (item in range)` loops
- [ ] Edge case: single-element range `a..a`, empty range `a>..<a`
- [ ] Integration test passes
