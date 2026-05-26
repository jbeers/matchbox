# R012: Add Template Interpolation And Script-Island CST Boundaries

**Type:** Tooling / Template parser correctness  
**Priority:** High  
**Related issues:** R004  
**Related files:** `crates/matchbox-compiler/src/tokenizer.rs`, `crates/matchbox-compiler/src/parser/template.rs`, `crates/matchbox-compiler/src/cst.rs`

## Current Status

Implemented on the current branch.

Completed on the current branch:

- `cst::parse_template()` round-trips exact source.
- Source gaps are preserved with `SyntaxElement::Source(Span)`.
- Existing template lexer tokens remain available through `SyntaxTree::tokens()`.
- `#expr#` is exposed as a structured interpolation CST node with nested script CST.
- `<bx:script>...</bx:script>` is exposed as a structured script-island CST node with nested script CST.
- Escaped hashes still round-trip distinctly from interpolation.

## Problem

The current template CST can preserve exact source, which is useful for round-trip safety, but formatter and LSP features need semantic template boundaries:

- output text
- component open/close tags
- component attributes
- interpolation start/end
- interpolation expression CST
- script island start/end
- script island body CST

Without these boundaries, tooling would need to re-scan `Source` gaps or call into the lossy runtime template parser.

## Solution

Build on the current lossless template CST:

1. Emit explicit tokens or CST elements for template delimiter spans that are currently source gaps.
2. Represent `#expr#` as a node containing start marker, script expression tokens/CST, and end marker.
3. Represent `<bx:script>...</bx:script>` as a node containing opening tag, nested script CST body, and closing tag.
4. Preserve `##` as escaped literal hash trivia/text.
5. Keep exact `to_source()` round-trip at every step.

## Test

```html
Hello <bx:output>#name#</bx:output>
<bx:script>
    var name = "World";
</bx:script>
```

Expected CST behavior:

- Template source round-trips exactly.
- `#name#` is exposed as an interpolation node.
- The interpolation expression exposes `name` as a script identifier token/node.
- The script island body exposes `var name = "World";` as nested script CST.

## Acceptance Criteria

- [x] Template delimiters are represented explicitly, not only as source gaps.
- [x] Interpolation regions expose expression boundaries.
- [x] Script islands expose nested script CST.
- [x] Escaped hashes round-trip and are distinguishable from interpolation.
- [x] Existing `parse_bxm()` runtime behavior is not regressed.
- [x] Tests assert both CST structure and exact source round-trip.
