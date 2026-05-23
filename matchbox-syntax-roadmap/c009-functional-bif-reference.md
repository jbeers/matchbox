# C009: Functional BIF Reference `::method`

**Type:** AFK  
**Blocked by:** None

## What to build

Compile `::method` syntax to a reference to a built-in function, which can be called, stored in a variable, or passed as a callback.

### Compiler
When the parser produces `Identifier("::methodName")`, the compiler should:
1. Recognize the `::` prefix
2. Look up the BIF name in the prelude/global scope
3. Emit a `CONSTANT` reference to the BIF function object

Alternatively, add a dedicated AST variant `ExpressionKind::FunctionalBIF { name }` and compile it to push a BIF reference onto the stack.

### VM
BIF references should be callable values. The VM's CALL opcode should handle both compiled functions and BIF references. If BIF references are stored as a special constant type, CALL needs to dispatch to the BIF.

### Test
```
var fn = ::ucase;
println(fn("hello")); // expect HELLO

var arr = ["a", "b", "c"];
var mapped = arrayMap(arr, ::ucase);
println(mapped[0]); // expect A
```

## Acceptance criteria
- [ ] `::methodName` compiles to a callable reference
- [ ] BIF reference can be stored in a variable
- [ ] BIF reference can be passed as a callback
- [ ] BIF reference can be called directly
- [ ] Unknown BIF name produces compile error
- [ ] Integration test passes
