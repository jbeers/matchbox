# T004: Template Comment Lexer Mode

**Type:** AFK  
**Blocked by:** T001 (Lexer mode stack)

## What to build

Implement the `TEMPLATE_COMMENT` and `TEMPLATE_COMMENT_QUIET` lexer modes for `<!--- ... --->` style comments.

### TEMPLATE_COMMENT mode
- Entered when `<!---` is seen in DEFAULT_TEMPLATE or TEMPLATE_COMPONENT_MODE
- Skips all content until `--->`
- Supports NESTED comments: `<!---` inside a comment pushes another TEMPLATE_COMMENT mode
- `--->` pops the current TEMPLATE_COMMENT mode
- All tokens in comment mode go to the hidden channel (not emitted)

### TEMPLATE_COMMENT_QUIET mode
- Same as TEMPLATE_COMMENT but used inside component tags
- Entered when `<!---` is seen in TEMPLATE_COMPONENT_MODE

### Key difference from script comments
Template comments use `<!---` / `--->` delimiters (NOT `//` or `/* */`).
They support nesting — `<!--- outer <!--- inner ---> still outer --->`.

## Delivery

Changes to `crates/matchbox-compiler/src/tokenizer.rs`.

## Acceptance criteria

- [ ] `<!--- comment --->` is skipped entirely in template mode
- [ ] Nested comments work: `<!--- outer <!--- inner ---> still outer --->`
- [ ] Comments inside component tags (`<bx:if <!--- note ---> cond>`) work
- [ ] `<!---` in script mode is NOT treated as a template comment (script mode only)
- [ ] Unit tests for basic and nested comments
