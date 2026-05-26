# T008: Template Script Island

**Type:** AFK  
**Blocked by:** T003 (Script island lexer) + T005 (Template parser foundation)

## What to build

Parse `<bx:script>...</bx:script>` islands within templates. Script islands embed full BoxLang script code inside a template, using the existing script parser.

### Syntax
```
<bx:script>
    // Full BoxLang script code here
    var x = 10;
    for (var i = 0; i < 5; i++) {
        writeOutput("Count: " & i & "<br>");
    }
</bx:script>
```

### Parsing
- Lexer: enters TEMPLATE_SCRIPT mode, switches to DEFAULT_SCRIPT
- Script content is parsed by the existing script parser (`parse()`)
- The resulting `Vec<Statement>` is wrapped in `StatementKind::ScriptIsland`
- After `</bx:script>`, parser returns to template statement parsing

### AST
- `StatementKind::ScriptIsland(Vec<Statement>)` — the script statements

### Compilation
- Script island statements are compiled inline (they run in the template's scope)
- Variables declared in script islands are accessible from template tags

## Delivery

Changes: `crates/matchbox-compiler/src/ast/mod.rs`, `crates/matchbox-compiler/src/parser/template.rs`

## Acceptance criteria

- [ ] `<bx:script> valid_boxlang_code </bx:script>` parses correctly
- [ ] Script content produces valid `Vec<Statement>` via standard parser
- [ ] Variables declared in script island accessible to subsequent template tags
- [ ] Multiple script islands in one template work
- [ ] Integration test: `.bxm` with script island
