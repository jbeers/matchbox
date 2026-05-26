# T013: Template function/argument

**Type:** AFK  
**Blocked by:** T005 (Template parser foundation)

## What to build

Parse `<bx:function>` and `<bx:argument>` tags for defining UDFs within templates.

### Syntax
```
<bx:function name="myFunction" [returnType="string"] [access="public"]>
    <bx:argument name="x" [type="string"] [required="true"] [default="hello"]>
    <bx:argument name="y">
    <bx:return result>
</bx:function>
```

### Parsing
- `<bx:function>`: Parse `name`, optional `returnType`, `access` attributes
- `<bx:argument>`: Parse `name`, optional `type`, `required`, `default` attributes
- Body is template statements
- Produce `StatementKind::FunctionDecl { name, params, body, ... }`

### Example
```html
<bx:function name="greet" returntype="string">
    <bx:argument name="person">
    <bx:return "Hello, " & person & "!">
</bx:function>
```

## Delivery

Changes: `crates/matchbox-compiler/src/parser/template.rs`

## Acceptance criteria

- [ ] `<bx:function name="...">` parses function declaration
- [ ] `<bx:argument>` inside function parses parameter
- [ ] `required`, `default`, `type` attributes on argument parse
- [ ] `returntype` and `access` on function parse
- [ ] Body of function parsed as template statements
- [ ] Integration test: `.bxm` with UDF in template
