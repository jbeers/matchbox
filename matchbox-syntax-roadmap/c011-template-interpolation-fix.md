# C011: Template #expr# Interpolation Boundary Tokens

**Type:** AFK — Small fix  
**Blocked by:** None

## Problem

When `<bx:output>Hello #name#!</bx:output>` is tokenized, the lexer emits:
1. ContentText("Hello ")
2. Identifier("name") — from #name#
3. ContentText("!")

The template parser sees expression tokens (Identifier) mixed with ContentText and has no way to distinguish them from script island content. The expression tokens are currently skipped.

## Solution

Add boundary tokens around `#expr#` interpolation in template output mode. The lexer should emit `InterpStart` before the expression and `InterpEnd` after.

### Token stream after fix
1. ContentText("Hello ")
2. InterpStart — marks start of expression
3. Identifier("name") — expression content
4. InterpEnd — marks end of expression
5. ContentText("!")

### Parser changes
In `parse_template_statement`, when `InterpStart` is seen:
1. Advance past InterpStart
2. Collect all expression tokens until InterpEnd
3. Parse collected tokens as script expression
4. Emit `BufferOutput(StringInterpolation([Text("Hello "), Expression(name), Text("!")]))`

Merge adjacent ContentText and interpolated expressions into a single StringInterpolation.

### Lexer changes
In `tokenize_template_content`, when `#expr#` is encountered in output mode:
1. Emit `InterpStart` token before parsing the expression
2. Parse expression tokens inline (existing logic works)
3. Emit `InterpEnd` token after the closing `#`

### Test
```html
<bx:output>Hello #name#!</bx:output>
```
With `name = "World"`, output buffer should contain `"Hello World!"`.

## Acceptance criteria
- [ ] InterpStart/InterpEnd tokens emitted around #expr# in output mode
- [ ] Template parser recognizes boundary and builds StringInterpolation
- [ ] Single BufferOutput per output block (merged text + expressions)
- [ ] Multiple #expr# segments in one output block work
- [ ] `##` in output body produces literal `#`
- [ ] `#expr#` outside output mode is literal text (unchanged)
- [ ] Unit test for template lexer boundary tokens
- [ ] Integration test verifies output buffer content
