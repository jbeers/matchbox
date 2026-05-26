# T002: Implement Component Name and Attribute Lexer Modes

**Type:** AFK  
**Blocked by:** T001 (Lexer mode stack)

## What to build

Implement the lexer modes for parsing component opening tags: `TEMPLATE_POSSIBLE_COMPONENT`, `TEMPLATE_COMPONENT_NAME`, `TEMPLATE_COMPONENT_MODE`, `TEMPLATE_ATTVALUE`, `TEMPLATE_UNQUOTED_VALUE`, and `TEMPLATE_END_COMPONENT`.

### TEMPLATE_POSSIBLE_COMPONENT mode
- After `<` in template mode
- `bx:` → push TEMPLATE_COMPONENT_NAME mode
- `/bx:` → push TEMPLATE_END_COMPONENT mode
- `<` → emit ContentText (nested `<`)
- `#` when in output mode → expression interpolation
- Any other char → emit ContentText, pop back

### TEMPLATE_COMPONENT_NAME mode
- After `<bx:`
- Read component name keyword
- Known keywords: `function`, `argument`, `return`, `if`, `else`, `elseif`, `set`, `try`, `catch`, `finally`, `import`, `while`, `break`, `continue`, `include`, `property`, `rethrow`, `throw`, `switch`, `case`, `defaultcase`, `output`, `script`, `loop`
- Emit `ComponentName(Keyword)` for known, `ComponentName(Identifier)` for custom
- Push TEMPLATE_COMPONENT_MODE after name

### TEMPLATE_COMPONENT_MODE mode
- Inside opening tag after component name
- Whitespace → skipped
- `<!---` → push TEMPLATE_COMMENT
- `=` → push TEMPLATE_ATTVALUE
- `/>` → self-close: emit ComponentSelfClose, pop modes
- `>` → end tag: emit ComponentClose, pop modes
- Anything else → emit AttributeName

### TEMPLATE_ATTVALUE mode
- After `=` in component mode
- `#` → start expression interpolation (push hashMode + DefaultScript)
- `"` → read quoted string value
- `'` → read single-quoted string value
- `>` → end tag (pop modes)
- `/>` → self-close
- Any other → push TEMPLATE_UNQUOTED_VALUE

### TEMPLATE_UNQUOTED_VALUE mode
- Bare attribute values
- Accumulate characters as UnquotedValuePart until whitespace or `>`/`/>`

### TEMPLATE_END_COMPONENT mode
- After `</bx:`
- Read closing component name
- After `>`, pop modes to return to template content

## Delivery

Changes to `crates/matchbox-compiler/src/tokenizer.rs`.

## Acceptance criteria

- [ ] `TEMPLATE_POSSIBLE_COMPONENT` correctly identifies `bx:` prefix
- [ ] `TEMPLATE_COMPONENT_NAME` recognizes all known component keywords
- [ ] `TEMPLATE_COMPONENT_MODE` parses attributes with `=` and `"` values
- [ ] `TEMPLATE_ATTVALUE` handles quoted, unquoted, and `#expr#` values
- [ ] `TEMPLATE_UNQUOTED_VALUE` accumulates bare values correctly
- [ ] `TEMPLATE_END_COMPONENT` matches closing tag to opening
- [ ] Self-closing tags (`<bx:set x=10 />`) tokenize correctly
- [ ] Unit tests for each mode
