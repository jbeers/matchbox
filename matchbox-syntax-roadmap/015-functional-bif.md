# 015: Add Functional BIF Reference `::method`

**Type:** AFK  
**Blocked by:** #003 (Wire New Parser)

## What to build

Add the `::method` functional BIF (Built-In Function) reference syntax through all compiler layers.

BoxLang syntax:
```
::ucase          // reference to the ucase BIF as a callable value
::arrayMap       // reference to arrayMap BIF
```

A `::name` expression evaluates to a reference to a built-in function, which can be passed as a callback, stored in a variable, or invoked directly: `::ucase("hello")`.

## Delivery

- **Parser:** Parse `::` token → identifier as a primary expression. This is a prefix atom at the same level as identifier or literal.
- **AST:** Add `ExpressionKind::FunctionalBIF { name: String }` variant.
- **Compiler:** Emit bytecode that resolves the named BIF at compile time and pushes a reference to it onto the stack. The reference should be callable like any function value.
- **Test:** Integration test passing `::ucase` to `arrayMap`, storing in a variable, and invoking directly.

## Acceptance criteria

- [ ] `::methodName` parses as an expression
- [ ] `::ucase("hello")` calls the BIF and returns `"HELLO"`
- [ ] `arrayMap(arr, ::ucase)` passes the BIF as a callback
- [ ] `var fn = ::ucase; fn("hello")` stores and invokes through variable
- [ ] Compile error for unknown BIF name
- [ ] Integration test passes
