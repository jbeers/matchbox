# C012: Template Script Island Compilation

**Type:** AFK — Small fix  
**Blocked by:** C011 (same boundary token design)

## Problem

When `<bx:script>var x = 1;</bx:script>` is tokenized, the lexer silently switches to DefaultScript mode. No tokens are emitted for the opening or closing tags. Script tokens appear raw in the stream without any boundary marker, making them indistinguishable from `#expr#` expression tokens.

The template parser skips these unknown tokens.

## Solution

With the boundary token design from C011, script islands can be handled distinctly. The lexer should emit `ScriptIslandStart` and `ScriptIslandEnd` tokens around the script island body.

### Token stream after fix
1. ScriptIslandStart
2. Var, Identifier("x"), Equal, Number("1"), Semicolon — script tokens
3. ScriptIslandEnd

### Parser changes
In `parse_template_statement`, when `ScriptIslandStart` is seen:
1. Advance past start token
2. Collect all script tokens until ScriptIslandEnd
3. Reconstruct source text from tokens
4. Parse using script parser (`crate::parser::parse()`)
5. Return resulting statements

### Lexer changes
In `tokenize_template_content`, when `<bx:script>` is detected:
1. Consume opening `>` (already done)
2. Push TemplateScript + DefaultScript modes
3. **Emit ScriptIslandStart token** (NEW)

In DefaultScript mode, when `</bx:script>` is detected:
1. **Emit ScriptIslandEnd token** (NEW)
2. Pop modes

### Test
```html
<bx:script>var name = "World";</bx:script>
<bx:output>Hello #name#!</bx:output>
```
Output: `"Hello World!"`

## Acceptance criteria
- [ ] ScriptIslandStart/End tokens emitted around script island body
- [ ] Template parser compiles script content inline
- [ ] Variables declared in script island accessible to subsequent template tags
- [ ] Multiple script islands in one template work
- [ ] Script island inside `<bx:output>` works
- [ ] Unit test for template script island parsing
- [ ] Integration test verifies script island execution
