# PRD: MatchBox Template Syntax Parity with BoxLang Core

**Status:** Draft  
**Target:** Bring MatchBox template parsing to full parity with BoxLang Core's template grammar.

---

## 1. Problem Statement

MatchBox currently handles templates via `parser/bxm.rs` — a **185-line regex transpiler** that converts `.bxm` files to BoxLang source, then feeds them through the normal script parser.

BoxLang Core uses a sophisticated **multi-mode ANTLR lexer** with 16 modes that handles scripting and templating interleaved at the character level. The grammar proper is hundreds of lines of `BoxGrammar.g4` dedicated to template rules. Template content (literal HTML, `<bx:tag>`, `#expr#` interpolation) is parsed directly into typed AST nodes (`BoxBufferOutput`, `BoxComponent`, `BoxScriptIsland`, `BoxSwitchBreakingCase`, etc.) and compiled to proper bytecode/Java.

MatchBox's regex transpiler supports only 4 tags: `<bx:output>`, `<bx:set>`, `<bx:if>`, `<bx:else>`, `<bx:elseif>`. Unknown tags are silently ignored. Template inclusion (`<bx:include>`), loops (`<bx:loop>`), script islands (`<bx:script>`), components, and tag-based try/catch/switch/function are all missing.

### Specific limitations of the regex approach

- **Cannot handle nested tags** — regex is not stateful and has no concept of tag nesting
- **No `<bx:script>` support** — cannot switch to script parsing mode mid-template
- **No expression attributes** — `#expr#` inside tag attributes is not handled
- **No custom components** — any tag beyond the 4 hardcoded ones is silently dropped
- **Fragile attribute parsing** — `attrs.trim()` used as raw expression text; no quoted string handling, no escape processing
- **No validation** — malformed tags silently produce broken output
- **No source maps** — template line numbers don't map to generated source

---

## 2. Proposed Solution

**Extend the hand-written lexer with template modes** to match BoxLang's lexer architecture, and **add template parsing to the recursive-descent parser**.

### Architecture

```
.bxm source
     │
     ▼
┌──────────────┐
│  Lexer         │ Template mode stack (DEFAULT_TEMPLATE_MODE, COMPONENT modes, etc.)
│  (tokenizer)   │ Emits template tokens: CONTENT_TEXT, COMPONENT_OPEN, COMPONENT_CLOSE,
│                │ ATTRIBUTE_NAME, ATTRIBUTE_VALUE, ICHAR, etc.
└──────┬───────┘
       │
       ▼
┌──────────────┐
│  Parser         │ Template statement parser (template_if, template_set, template_output, etc.)
│  (parser)       │ Produces new AST node types: BoxBufferOutput, BoxComponent, BoxScriptIsland
│                 │ Expression interpolation inside templates produces StringInterpolation
└──────┬───────┘
       │
       ▼
┌──────────────┐
│  Compiler       │ Template-specific compilation: writeToBuffer, component invocation
│  (compiler)     │ Script islands compiled inline, output blocks generate output ops
│                 │ → Chunk (bytecode)
└──────────────┘
```

### Lexer Mode Architecture

BoxLang uses 16 modes. For MatchBox, we need approximately 8-10 modes focused on the template path (script-mode string/hash modes are already handled):

| Mode | Purpose |
|---|---|
| `DEFAULT_TEMPLATE` | Primary mode for `.bxm` files. Emits CONTENT_TEXT, detects `<bx:tag` starts, handles `#expr#` |
| `TEMPLATE_POSSIBLE_COMPONENT` | After `<`, determines if this is a component or literal text |
| `TEMPLATE_COMPONENT_NAME` | After `<bx:`, reads the component name |
| `TEMPLATE_COMPONENT_MODE` | Inside the opening tag, parses attributes |
| `TEMPLATE_ATTVALUE` | Inside an attribute value (`name="value"`, `name=#expr#`, etc.) |
| `TEMPLATE_UNQUOTED_VALUE` | Bare attribute values like `foo=bar` |
| `TEMPLATE_OUTPUT_MODE` | Marker mode — enables `#expr#` interpolation in body content |
| `TEMPLATE_END_COMPONENT` | After `</bx:`, reads closing component name |
| `TEMPLATE_COMMENT` | Inside `<!--- ... --->` comments |
| `TEMPLATE_SCRIPT` | Marker mode — inside `<bx:script>...</bx:script>`, parsing in DEFAULT_SCRIPT |

### Tags to Support

All tags from BoxLang's template grammar:

| Tag | Priority | Notes |
|---|---|---|
| `<bx:if>` / `<bx:elseif>` / `<bx:else>` | High | Already partially supported via BXM |
| `<bx:set>` | High | Already partially supported via BXM |
| `<bx:output>` | High | Already partially supported via BXM |
| `<bx:loop>` | High | Core iteration tag — currently missing |
| `<bx:return>` | High | Return from template function |
| `<bx:include>` | Medium | Template inclusion |
| `<bx:try>` / `<bx:catch>` / `<bx:finally>` | Medium | Error handling in templates |
| `<bx:switch>` / `<bx:case>` / `<bx:defaultcase>` | Medium | Multi-way branching |
| `<bx:script>` | Medium | Inline script islands |
| `<bx:while>` | Medium | Loop with condition attribute |
| `<bx:function>` / `<bx:argument>` | Medium | UDFs in templates |
| `<bx:throw>` / `<bx:rethrow>` | Low | Exception throwing in templates |
| `<bx:import>` | Low | Import directives in templates |
| `<bx:break>` / `<bx:continue>` | Low | Loop control in template loops |
| `<bx:property>` | Low | Component property declarations |
| Generic `<bx:custom>` components | Low | Extensible component system |
| `` ```...``` `` component islands | Low | Template within script |

---

## 3. AST Changes

New AST nodes needed:

| Node | Purpose |
|---|---|
| `StatementKind::BufferOutput(Expression)` | Emits content to the output buffer |
| `StatementKind::Component { name, attributes, body }` | Generic component tag |
| `StatementKind::ScriptIsland(Vec<Statement>)` | Wraps `<bx:script>` content |
| `StatementKind::TemplateIsland(Vec<Statement>)` | Wraps `` ```...``` `` content |
| `ExpressionKind::StringInterpolation(Vec<StringPart>)` | Already exists as `Literal::String(Vec<StringPart>)` |

Template-specific statement variants use existing script AST nodes where possible:
- `template_if` → `StatementKind::If`
- `template_set` → `StatementKind::Expression` (assignment)
- `template_loop` → `StatementKind::ForLoop`
- `template_switch` → `StatementKind::Switch`
- `template_try` → `StatementKind::TryCatch`
- `template_return` → `StatementKind::Return`
- `template_function` → `StatementKind::FunctionDecl`

---

## 4. Compiler Changes

| Feature | Compilation Strategy |
|---|---|
| `BufferOutput` | Emit `writeToBuffer(expr)` opcode or equivalent |
| `#expr#` interpolation in text | Compile as `StringConcat` of literal parts + expression evaluations |
| `<bx:output>` body | Enable `#expr#` interpolation; compile text + expressions as output |
| `<bx:script>` island | Compile statements inline (same as regular script) |
| Components | Invoke component handler at runtime (emit function call) |
| Source maps | Track template source lines → generated bytecode positions |

---

## 5. Out of Scope

- **CFML template dialect** (`CFLexer.g4` + `CFGrammar.g4` + CFML compatibility mode)
- **Custom component system** — BoxLang's `ComponentService` with dynamic component registration
- **Component cache** — Pre-compiled component templates
- **Template encryption** / obfuscation
- **tag-based outputting components** beyond `<bx:output>`

---

## 6. Implementation Phases

### Phase 1: Lexer Template Modes (foundation)
Implement the mode stack and all template-specific lexer modes. This is the critical path — the parser can't do anything without template tokens.

### Phase 2: Core Template Parsing
Parse literal text, `<bx:output>`, `<bx:set>`, `<bx:if>/<bx:elseif>/<bx:else>`, `<bx:loop>`, `<bx:return>`, `<bx:script>`. Replace the BXM transpiler with direct template parsing.

### Phase 3: Extended Template Tags
`<bx:include>`, `<bx:try>/<bx:catch>/<bx:finally>`, `<bx:switch>/<bx:case>/<bx:defaultcase>`, `<bx:while>`, `<bx:function>/<bx:argument>`, `<bx:throw>/<bx:rethrow>`, `<bx:import>`, `<bx:break>/<bx:continue>`, `<bx:property>`.

### Phase 4: Components and Islands
Generic `<bx:custom>` components, `` ```...``` `` island syntax within scripts.
