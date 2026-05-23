# T003: Implement Script Island and Output Mode Lexer

**Type:** AFK  
**Blocked by:** T002 (Component tag lexer)

## What to build

Implement `<bx:script>...</bx:script>` island lexing and the `TEMPLATE_OUTPUT_MODE` for `#expr#` interpolation inside `<bx:output>` bodies.

### TEMPLATE_SCRIPT mode
- A marker mode pushed when entering `<bx:script>` body
- Script content is parsed in DEFAULT_SCRIPT mode
- In DEFAULT_SCRIPT mode, `</bx:script>` is detected and pops both TEMPLATE_SCRIPT and DEFAULT_SCRIPT modes, returning to DEFAULT_TEMPLATE

### TEMPLATE_OUTPUT_MODE
- A marker mode pushed when entering `<bx:output>` body
- Enables `#expr#` interpolation in the body content
- In DEFAULT_TEMPLATE mode, when `#` is encountered AND `TEMPLATE_OUTPUT_MODE` is on the stack, start expression interpolation
- When NOT in output mode, `#` is emitted as literal ContentText
- `##` inside output mode → ContentText with `#` (escaped)
- `#expr#` → ICHAR + expression tokens + ICHAR

### String/Hash Mode Integration
- The existing `hashMode` (from script string interpolation) is reused
- When `#` starts an expression in template, push `hashMode` + `DEFAULT_SCRIPT`
- Parse expression tokens in script mode
- When closing `#` is found, pop back to template mode
- Emit ICHAR tokens at boundaries

## Delivery

Changes to `crates/matchbox-compiler/src/tokenizer.rs`.

## Acceptance criteria

- [ ] `<bx:script> code </bx:script>` produces script-level tokens inside
- [ ] After `</bx:script>`, lexer returns to template mode
- [ ] TEMPLATE_OUTPUT_MODE enables `#expr#` in body content
- [ ] `#` outside output mode is literal text
- [ ] `##` inside output mode produces single `#`
- [ ] `#expr#` correctly tokenizes the expression with ICHAR boundaries
- [ ] Expression tokens inside `#expr#` parse correctly (identifiers, operators, function calls)
- [ ] Unit tests for each mode
