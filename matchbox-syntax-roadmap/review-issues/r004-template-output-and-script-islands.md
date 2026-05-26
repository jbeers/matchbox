# R004: Make Template Output And Script Islands Executable

**Type:** Correctness / Template compatibility  
**Priority:** High  
**Related issues:** `c011-template-interpolation-fix.md`, `c012-template-script-island-fix.md`
**Related files:** `crates/matchbox-compiler/src/tokenizer.rs`, `crates/matchbox-compiler/src/parser/template.rs`

## Status

Resolved in code. Template output now preserves literal text, evaluates `#expr#` interpolation, and executes `<bx:script>` islands in sequence.

## Problem

Template parsing was previously mostly parse-only for important runtime behavior:

- Plain `ContentText` was trimmed, which changed output.
- `#expr#` interpolation tokens were skipped by the parser.
- `<bx:script>` bodies were skipped.
- Tests mostly asserted parsing succeeded rather than output content.

This blocks BXM compatibility because templates need to preserve text, evaluate interpolation, and execute script islands in sequence.

## Solution

Complete the deferred C011/C012 behavior:

1. Emit `InterpStart` and `InterpEnd` around output-mode interpolation.
2. Parse content plus interpolation into one `Literal::String` with `StringPart::Text` and `StringPart::Expression`.
3. Preserve raw output text unless whitespace trimming is explicitly requested by syntax.
4. Emit `ScriptIslandStart` and `ScriptIslandEnd`, or otherwise collect script body tokens safely.
5. Parse script islands with the script parser and splice resulting statements inline.

## Test

```html
<bx:script>
    var name = "World";
</bx:script>
Hello #name#!
```

Expected output:

```text
Hello World!
```

## Acceptance Criteria

- [x] Template text is not trimmed unexpectedly.
- [x] `#expr#` emits evaluated output in output mode.
- [x] Multiple interpolation segments work in one output block.
- [x] `##` emits a literal `#`.
- [x] Script island variables are visible to later template output.
- [x] Integration tests assert exact output buffer content.
