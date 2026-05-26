# C012: Template Script Island and Component Compilation

**Type:** AFK  
**Blocked by:** C006 (Output buffer), C011 (Template interpolation)

## What to build

Implement proper compilation for template script islands, `<bx:loop>`, `<bx:include>`, `<bx:function>`, and generic components.

### Script Island (`<bx:script>`)
- Script statements compile inline in the template's scope
- Variables declared in script islands are accessible to subsequent template tags
- The lexer already handles tokenization correctly

### `<bx:loop>`
- Currently desugared to WhileLoop
- For array loops: compile as for-in with array expression
- For condition loops: compile as while loop

### `<bx:include>`
- Same as script `include` statement
- Load, parse, compile, execute inline

### `<bx:function>`
- Compile template UDF as a regular function
- Body is compiled as template statements (output goes to buffer)
- Function is registered in the template scope

### Generic Components
- `StatementKind::Component { name, attributes, body }`
- For now, skip compilation (emit no-op) — full component system needs runtime component registry

### Test
```
<bx:script>
    var name = "World";
</bx:script>
<bx:output>Hello #name#!</bx:output>
// Output: "Hello World!"
```

## Acceptance criteria
- [ ] Script island variables accessible to template tags
- [ ] `<bx:loop array="#arr#" item="val">` iterates correctly
- [ ] `<bx:include template="...">` loads and executes
- [ ] `<bx:function>` compiles as callable function
- [ ] Integration tests for each feature
