# C008: Spread Expression Desugaring

**Type:** AFK  
**Blocked by:** None

## What to build

Implement compile-time desugaring of spread expressions (`...expr`) in function call arguments, array literals, and struct literals.

### Function call spread: `func(a, ...arr, b)`
Desugar to:
1. Push `a`
2. Iterate `arr`, push each element
3. Push `b`
4. CALL func with N args

Compiler must track argument count correctly.

### Array literal spread: `[a, ...arr, b]`
Desugar to:
1. Create new array
2. Push `a`
3. Iterate `arr`, push each element
4. Push `b`

### Struct literal spread: `{a: 1, ...obj, b: 2}`
Desugar to:
1. Create new struct
2. Set key `a` = 1
3. Iterate `obj` keys, set each key = value
4. Set key `b` = 2

### Test
```
var arr = [2, 3];
var result = [1, ...arr, 4];
println(len(result)); // expect 4

var obj = {b: 2, c: 3};
var merged = {a: 1, ...obj, d: 4};
println(merged.a); // expect 1
println(merged.b); // expect 2
println(merged.c); // expect 3
println(merged.d); // expect 4
```

## Acceptance criteria
- [ ] Spread in function arguments works
- [ ] Spread in array literals works
- [ ] Spread in struct literals works
- [ ] Mixed spread and regular elements works
- [ ] Spread of empty array/struct works
- [ ] Integration test passes
