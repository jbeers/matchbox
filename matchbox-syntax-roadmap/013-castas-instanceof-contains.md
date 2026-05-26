# 013: Add `castAs`, `instanceOf`, `contains`/`doesNotContain`

**Type:** AFK  
**Blocked by:** #003 (Wire New Parser)

## What to build

Add type-check and containment operators through all compiler layers.

BoxLang operators:
```
expr castAs "TypeName"   — cast expression to type
expr instanceOf TypeName  — type check, returns boolean
expr contains expr         — containment check (string/array/struct)
expr does not contain expr — negated containment check
```

## Delivery

- **Lexer:** Already handled by the keyword tokenizer in #001. Ensure `castAs`, `instanceOf`, `contains` are tokenized as keywords. `does not contain` is a three-word operator — the lexer should produce a single `DoesNotContain` token or the parser should handle the two-keyword sequence.
- **Parser:** Add these as binary operators in the Pratt parser:
  - `castAs`: precedence between equality and addition
  - `instanceOf`: same as comparison
  - `contains`: same as comparison
  - `does not contain`: same as comparison, negated
- **AST:** Add `ExpressionKind::CastAs { expr, type }` and reuse `Binary` for `instanceOf`, `contains`, `notContains` (or add dedicated variants).
- **Compiler:** 
  - `castAs`: emit runtime type-cast call
  - `instanceOf`: emit runtime type-check call
  - `contains`: emit runtime containment check (delegates to type-specific logic)
  - `does not contain`: emit `contains` + negate
- **Test:** Integration test for each operator.

## Acceptance criteria

- [ ] `expr castAs "string"` parses and runs
- [ ] `expr instanceOf SomeType` parses and runs, returning boolean
- [ ] `"hello" contains "ell"` returns true
- [ ] `"hello" does not contain "xyz"` returns true
- [ ] `[1,2,3] contains 2` returns true
- [ ] `{a:1} contains "a"` returns true (struct key check)
- [ ] Integration test passes
