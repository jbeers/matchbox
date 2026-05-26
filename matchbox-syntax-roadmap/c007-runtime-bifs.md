# C007: Runtime BIFs — castAs, instanceOf, contains

**Type:** AFK  
**Blocked by:** None

## What to build

Implement runtime built-in functions for `castAs`, `instanceOf`, and `contains`. These are word operators that the parser already tokenizes — the compiler just needs to emit calls to runtime BIFs.

### Approach
Since the compiler treats these as opcode `"instanceof"`, `"castas"`, `"contains"` in binary expressions, the simplest approach is to handle them as regular binary ops in the VM or emit function calls.

Better approach: In the compiler, when the operator is `"instanceof"`, `"castas"`, or `"contains"`, emit a function call to the corresponding BIF:
- `a instanceOf B` → `CALL instanceOf(a, str(B))`
- `a castAs B` → `CALL castAs(a, str(B))`
- `a contains b` → `CALL contains(a, b)`

### BIF implementations (in prelude or native)
- `castAs(value, typeName)` — attempt type conversion
- `instanceOf(value, typeName)` — check if value is instance of type
- `contains(collection, item)` — for strings: substring check; for arrays: element check; for structs: key check

### Test
```
println("hello" contains "ell");     // expect true
println("hello" contains "xyz");     // expect false
println([1,2,3] contains 2);        // expect true
println({a:1} contains "a");        // expect true
println(42 instanceof "numeric");   // expect true (numeric type check)
```

## Acceptance criteria
- [ ] `contains` works for strings, arrays, and structs
- [ ] `does not contain` is `NOT contains(...)` (negation handled by parser)
- [ ] `instanceOf` performs type checking
- [ ] `castAs` performs type conversion where possible
- [ ] Integration test passes
