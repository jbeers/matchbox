# T001: Implement Lexer Mode Stack + DEFAULT_TEMPLATE Mode

**Type:** AFK  
**Blocked by:** None — can start immediately

## What to build

Add a mode stack to the hand-written lexer and implement the `DEFAULT_TEMPLATE` mode. This is the foundational piece for all template parsing.

The lexer currently only has a single mode (script mode). It needs:
- A `LexerMode` enum with variants: `DefaultScript`, `DefaultTemplate`, `TemplatePossibleComponent`, `TemplateComponentName`, `TemplateComponentMode`, `TemplateAttrValue`, `TemplateUnquotedValue`, `TemplateOutput`, `TemplateEndComponent`, `TemplateComment`, `TemplateScript`
- A mode stack (`Vec<LexerMode>`) in the `Lexer` struct
- Methods: `push_mode()`, `pop_mode()`, `current_mode()`, `mode_stack_contains(mode)`
- Mode-aware dispatch in the main tokenize loop

### DEFAULT_TEMPLATE mode behavior

- Emit `CONTENT_TEXT` token for literal text (anything that's not `<`, `#`, or backtick)
- On `<!---`, push `TEMPLATE_COMMENT` mode
- On `##`, emit `CONTENT_TEXT` with value `#` (escaped hash)
- On `#` (when in output mode — check mode stack), start expression interpolation: push `hashMode` marker, parse expression tokens, emit `ICHA` start/end markers
- On `` ``` `` (when in component island), pop out of component island
- On `<bx:script...>`, push `TEMPLATE_SCRIPT` + switch to `DEFAULT_SCRIPT` mode
- On `<bx:output`, push `TEMPLATE_POSSIBLE_COMPONENT` + component + output modes
- On `<`, push `TEMPLATE_POSSIBLE_COMPONENT`
- Everything else: emit `CONTENT_TEXT`

### New token kinds needed

- `ContentText` — literal text content
- `ComponentOpen` — `<` that starts a component
- `ComponentClose` — `>` that closes a component
- `ComponentSelfClose` — `/>`
- `AttributeName` — attribute name in a tag
- `AttributeValue` — attribute value
- `UnquotedValuePart` — part of a bare attribute value
- `Ichar` — `#` marking interpolation start/end
- `ScriptOpen` — `<bx:script` boundary
- `ScriptClose` — `</bx:script>` boundary

## Delivery

Changes to `crates/matchbox-compiler/src/tokenizer.rs`:
- `LexerMode` enum
- Mode stack on `Lexer`
- DEFAULT_TEMPLATE mode implementation
- New token kinds in `TokenKind`

## Acceptance criteria

- [ ] Mode stack API: push, pop, current, contains
- [ ] DEFAULT_TEMPLATE mode emits ContentText for literal text
- [ ] `<` pushes TEMPLATE_POSSIBLE_COMPONENT
- [ ] `##` → ContentText with `#`
- [ ] `<!---` → TEMPLATE_COMMENT mode (comment content skipped)
- [ ] Tokenizer unit tests for template content
- [ ] Default script mode still works when mode stack is empty
