# T006: Template if/elseif/else

**Type:** AFK  
**Blocked by:** T005 (Template parser foundation)

## What to build

Parse `<bx:if>`, `<bx:elseif>`, and `<bx:else>` tags into `StatementKind::If` AST nodes.

### Syntax
```
<bx:if expression>
    body
[<bx:elseif expression>
    body]*
[<bx:else>
    body]
</bx:if>
```

### Parsing
- `<bx:if>`: The expression after `if` is parsed as a script expression (in `DEFAULT_SCRIPT` mode via expression-in-tag handling)
- Body is a sequence of template statements
- `<bx:elseif>`: Chain as nested `If` in the else branch
- `<bx:else>`: Simple else branch
- Closing `</bx:if>`: end of the if block

### Example
```html
<bx:if x GT 10>
    <bx:output>Big: #x#</bx:output>
<bx:elseif x GT 5>
    <bx:output>Medium: #x#</bx:output>
<bx:else>
    <bx:output>Small: #x#</bx:output>
</bx:if>
```

Produces: `If { condition: Binary(x > 10), then: [BufferOutput(...)], else: [If { condition: Binary(x > 5), then: [...], else: [...] }] }`

## Delivery

Changes: `crates/matchbox-compiler/src/parser/template.rs`

## Acceptance criteria

- [ ] `<bx:if condition> body </bx:if>` parses correctly
- [ ] `<bx:elseif condition>` chains as nested else-if
- [ ] `<bx:else>` provides else branch
- [ ] Expression inside `<bx:if>` is parsed as full script expression
- [ ] Nested `<bx:if>` inside `<bx:if>` works
- [ ] Integration test: `.bxm` with if/elseif/else
- [ ] Existing BXM if/else tests pass
