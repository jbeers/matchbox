# T010: Template try/catch/finally

**Type:** AFK  
**Blocked by:** T005 (Template parser foundation)

## What to build

Parse `<bx:try>`, `<bx:catch>`, and `<bx:finally>` tags into `StatementKind::TryCatch` AST nodes.

### Syntax
```
<bx:try>
    body
[<bx:catch [type="ExceptionType"]>
    body]*
[<bx:finally>
    body]
</bx:try>
```

### Parsing
- `<bx:try>`: Begin try block, body is template statements
- `<bx:catch>`: Optional `type` attribute for exception type filtering, optional `variable` attribute for exception variable name
- `<bx:finally>`: Finally block body
- Map to `StatementKind::TryCatch { try_branch, catches, finally_branch }`

### Example
```html
<bx:try>
    <bx:output>#riskyOperation()#</bx:output>
<bx:catch type="DatabaseException">
    <bx:output>DB error occurred</bx:output>
<bx:catch>
    <bx:output>Unknown error</bx:output>
<bx:finally>
    <bx:output>Cleanup done</bx:output>
</bx:try>
```

## Delivery

Changes: `crates/matchbox-compiler/src/parser/template.rs`

## Acceptance criteria

- [ ] `<bx:try> body <bx:catch> body </bx:try>` parses correctly
- [ ] Multiple `<bx:catch>` blocks parse
- [ ] `<bx:catch type="...">` parses type filter
- [ ] `<bx:finally>` block parses
- [ ] Integration test: `.bxm` with try/catch/finally
