# C004: Fix Statement Semantics — assert, param, rethrow

**Type:** AFK  
**Blocked by:** None

## What to build

Implement proper runtime behavior for `assert`, `param`, and `rethrow` statements.

### assert
- Compile condition expression
- If truthy: POP and continue
- If falsy: throw `AssertError` with optional message
- Currently emits JUMP_IF_FALSE but doesn't complete the falsy path (no throw)

**Fix:** Complete the bytecode pattern — compile condition, JUMP_IF_FALSE to throw path, POP (truthy), JUMP to end, throw path: POP + CONSTANT(message) + THROW.

### param
- `param name = default` — if variable `name` is undefined or null, set it to `default`
- `param name` (no default) — if variable `name` is undefined or null, throw error

**Fix:** GET_GLOBAL(name) → JUMP_IF_NULL → skip path (POP + end), null path (POP + CONSTANT(default) + SET_GLOBAL(name)). If no default, null path throws.

### rethrow
- Only valid inside `catch` block
- Re-throws the currently caught exception
- Preserves original stack trace

**Fix:** Add `RETHROW` opcode. VM tracks current exception in catch handler. RETHROW pops handler and re-throws.

### VM opcodes
- `RETHROW` (77) — re-throws current exception from catch context

### Test
```
// assert
assert true;
try { assert false : "failed"; } catch (e) { println(e.message); } // expect "failed"

// param
param x = 42;
println(x); // expect 42
param x = 99;
println(x); // expect 42 (already set)
try { param y; } catch (e) { println("param error"); } // expect param error

// rethrow
try { try { throw "inner"; } catch (e) { rethrow; } } catch (e) { println("caught: " & e.message); }
```

## Acceptance criteria
- [ ] Passing assert is a no-op
- [ ] Failing assert throws AssertError with message
- [ ] param with default sets value only if undefined/null
- [ ] param without default throws if undefined/null
- [ ] rethrow inside catch re-throws the caught exception
- [ ] rethrow outside catch produces compile error
- [ ] Integration test passes
