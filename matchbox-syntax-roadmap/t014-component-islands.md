# T014: Component Islands (``` ```)

**Type:** AFK  
**Blocked by:** T005 (Template parser foundation)

## What to build

Support component island syntax (`` ```...``` ``) within script files. This is the inverse of script islands — it allows embedding template code inside `.bxs` script files.

### Syntax
```
// This is a .bxs script file
var name = "World";
```

<bx:output>
    <h1>Hello #name#</h1>
</bx:output>
```

var more = "script code";
```

### Lexer
- In DEFAULT_SCRIPT mode, `` ``` `` (triple backtick) pushes `componentIsland` + `DEFAULT_TEMPLATE` modes
- Template content is parsed in DEFAULT_TEMPLATE mode
- Closing `` ``` `` (only when `componentIsland` mode is on the stack) pops back to script mode

### Parser
- Component island content produces template AST (BufferOutput, Component, etc.)
- Wrapped in `StatementKind::TemplateIsland(Vec<Statement>)`

### Inception
Script islands can contain component islands which can contain script islands, ad infinitum. The mode stack handles this naturally.

## Delivery

Changes: `crates/matchbox-compiler/src/tokenizer.rs`, `crates/matchbox-compiler/src/ast/mod.rs`, `crates/matchbox-compiler/src/parser/`

## Acceptance criteria

- [ ] `` ```...``` `` in `.bxs` file switches to template mode
- [ ] Template content inside island parses correctly
- [ ] Closing `` ``` `` returns to script mode
- [ ] Component island inside script island works
- [ ] Integration test: `.bxs` with component island
