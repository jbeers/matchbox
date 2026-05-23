# C011: Template Expression Interpolation in Output Mode

**Type:** AFK  
**Blocked by:** C006 (Output buffer support)

## What to build

Implement proper `#expr#` interpolation inside `<bx:output>` bodies. Currently the lexer tokenizes the expression tokens but the template parser doesn't assemble them into string interpolation output.

### Parser changes
When parsing `<bx:output>` body content, collect alternating ContentText and expression tokens. Merge them into a single `BufferOutput` statement with a `StringInterpolation` expression:
- `ContentText("Hello ")` + `Identifier("name")` + `ContentText("!")`
- → `BufferOutput(StringInterpolation([Text("Hello "), Expression(name), Text("!")]))`

### Compiler
Compile `StringInterpolation` by evaluating each part and concatenating:
1. For each text part: push string literal
2. For each expression part: compile and push result (converted to string)
3. Concatenate all parts
4. Emit BUFFER_WRITE

### Test
```
<bx:output>Hello #name#!</bx:output>
// With name = "World", outputs "Hello World!"
```

### Existing test
The `test_nested_bxm_interpolation` test in web_runtime_tests.rs currently just verifies parsing doesn't error. It should be updated to verify correct output.

## Acceptance criteria
- [ ] `#expr#` in `<bx:output>` body evaluates expression
- [ ] Literal text + `#expr#` + literal text produces correct concatenated output
- [ ] Multiple `#expr#` segments in one output block work
- [ ] `##` in output body produces literal `#`
- [ ] `#expr#` outside `<bx:output>` is literal text (not evaluated)
- [ ] Integration test verifies output buffer content
