# 016: Add Spread Expression `...expr`

**Type:** AFK  
**Blocked by:** #003 (Wire New Parser)

## What to build

Add the spread (`...`) expression through all compiler layers. The spread operator expands an array or struct into individual elements at the call site or literal construction site.

BoxLang syntax:
```
// Spread in function calls
func(...myArray)
func(a, ...myArray, b)
func(...myStruct)      // spreads key=value pairs as named arguments

// Spread in array literals
[ a, ...myArray, b ]

// Spread in struct literals
{ a: 1, ...myStruct, b: 2 }
```

## Delivery

- **Parser:** Parse `...` token → expression in three contexts:
  1. Function call arguments: `spreadArgument` in argument list
  2. Array literal members: `...expr` alongside regular expressions
  3. Struct literal members: `...expr` as a member (no key)
- **AST:** Add `ExpressionKind::Spread(Box<Expression>)` variant. Use it in argument lists, array members, and struct members via an enum or by allowing `Spread` in those positions.
- **Compiler:** Emit bytecode that, at runtime:
  - For function call spread: unpack the array/struct and push elements/pairs onto the call stack
  - For array literal spread: iterate the source array and append elements
  - For struct literal spread: iterate the source struct and merge key-value pairs
- **Test:** Integration test for each spread context, including spreading empty collections and mixing spread with regular elements.

## Acceptance criteria

- [ ] `func(...arr)` spreads array elements as positional arguments
- [ ] `func(...struct)` spreads struct as named arguments
- [ ] `[a, ...arr, b]` spreads into array literal
- [ ] `{a: 1, ...struct, b: 2}` spreads into struct literal
- [ ] Spread of empty array/struct works correctly
- [ ] Mixing spread and regular args/elements works
- [ ] Integration test passes
