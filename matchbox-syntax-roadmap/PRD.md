# PRD: MatchBox Syntax Parity with BoxLang Core

**Status:** Approved  
**Target:** Bring MatchBox's compiler to full syntax parity with BoxLang Core (BoxLang native dialect only; CFML compat and SQL are out of scope for this PRD).

---

## 1. Problem Statement

MatchBox currently uses a single **pest** PEG grammar (`boxlang.pest`, 229 lines) to parse BoxLang source. This grammar implements approximately 40–50% of BoxLang's native language surface area. The parser builds a small AST (16 statement kinds, 14 expression kinds) then emits bytecode directly into a custom stack-based VM.

BoxLang Core uses **ANTLR** with 8 grammar files, a rich lexer with 16 modes, and parses the full BoxLang native language. MatchBox is missing major expression features (bitwise ops, ranges, `castAs`, `instanceOf`, `contains`, destructuring, functional BIF references, spread), several statement types (`do/while`, `assert`, `param`, `rethrow`, `include`), and has no template parsing beyond a basic `.bxm` regex transpiler.

The pest approach has structural limitations:
- **No support for ANTLR-style lexer modes** — pest is a scannerless PEG. Implementing BoxLang's 16-mode lexer in pest would require manual state-tracking hacks.
- **No left recursion** — expressions must be parsed with a separate Pratt parser bolted on, duplicating operator definitions.
- **Heavy intermediate representation** — pest builds a full parse tree before AST construction, doubling allocations.
- **Limited error recovery** — pest has minimal error recovery support compared to ANTLR's built-in strategies.

---

## 2. Proposed Solution

**Replace pest with a hand-written recursive descent parser.**

### Why recursive descent over pest or ANTLR4Rust:

| | pest | ANTLR4Rust | Hand-written |
|---|---|---|---|
| Lexer modes | Must emulate manually | Built-in, matches BoxLang | Full control, straightforward state machine |
| Left recursion | Not supported; needs Pratt bolt-on | Native ANTLR support | Standard Pratt in code |
| Parse tree overhead | Full intermediate tree | Full intermediate tree | Direct AST construction, zero intermediate |
| Rust ecosystem | Good (pure Rust) | Weak (`antlr4rust` niche, adds Java build dep) | Best — no external deps |
| Error recovery & reporting | Minimal | ANTLR error strategies | Full control over errors |
| Grammar sync with BoxLang `.g4` | Must manually rewrite | Could derive/port from `.g4` | Must manually translate, but in idiomatic Rust |
| Allocations | Pair-per-node | ParseTree-per-node | Bump arena, one-drop free |
| Performance | Moderate | Moderate | Fastest (rustc/SWC/oxc pattern) |
| Binary size | Small + pest runtime | Heavy (ANTLR runtime) | Minimal |

**Decision:** Hand-written recursive descent with a separate lexer (tokenizer), followed by a Pratt expression parser. This is the standard approach for production Rust language tools (rustc, SWC, oxc, biome, deno).

### Architecture

```
Source text
     │
     ▼
┌──────────────┐
│  Lexer        │  Tokenizer with modes (script/template)
│  (tokenizer)  │  → Vec<Token> (with span info)
└──────┬───────┘
       │
       ▼
┌──────────────┐
│  Parser       │  Recursive descent for statements
│  (parser)     │  Pratt parser for expressions
│               │  → AST (Vec<Statement>)
└──────┬───────┘
       │
       ▼
┌──────────────┐
│  Compiler     │  Existing compiler (largely unchanged)
│  (compiler)   │  New AST node kinds handled here
│               │  → Chunk (bytecode)
└──────────────┘
```

The **AST** and **compiler** are kept mostly intact. The parser is a drop-in replacement for the current `parse()` function — it reads source text and produces `Vec<Statement>`. The new parser lives in a new crate or module alongside the existing one, allowing incremental cutover.

---

## 3. Scope — What Must Be Supported

### 3.1 Phase 1: Custom Parser (foundation)

Replace pest with hand-written lexer+parser. **Feature-complete reimplementation of all currently supported syntax**, plus structural support for future additions.

- [ ] Tokenizer with: identifiers, keywords, literals (string, number, boolean, null), operators, punctuation
- [ ] Lexer modes for string interpolation (`#expr#`)
- [ ] Recursive descent statement parser
- [ ] Pratt expression parser with all current operators
- [ ] AST construction producing identical `Vec<Statement>` output
- [ ] Source-span tracking on AST nodes (for error reporting)
- [ ] Error recovery and multi-error collection
- [ ] Remove `pest` and `pest_derive` from Cargo.toml

### 3.2 Phase 2: Statement Parity

Statements present in BoxLang but missing from MatchBox:

| Feature | BoxLang Grammar | Priority |
|---|---|---|
| `do { } while(cond);` | `do` rule line 336 | High |
| `assert expr;` / `assert expr : "msg";` | `assert` rule line 349 | High |
| `param type name = val;` | `param` rule line 303 | High |
| `rethrow;` | `rethrow` rule line 368 | High |
| `include "file.bxm";` | `include` rule line 62 | Medium |
| `break label;` / `continue label;` | lines 353, 358 | Medium |
| `not (expression)` as statement | `not` rule line 256 | Medium |
| `static { ... }` initializer | `staticInitializer` rule line 83 | Medium |
| `PACKAGE`, `REMOTE` access modifiers | `accessModifier` rule line 112 | Medium (compiler work too) |
| `DEFAULT`, `STATIC`, `ABSTRACT`, `FINAL` modifiers | `modifier` rule line 104 | Medium (compiler work too) |
| `local class` (named class in script) | `localClass` rule line 73 | Low (compiler work dominant) |
| `postAnnotation` on functions | `postAnnotation` rule line 96 | Low |
| Properties with annotations (`@foo property name;`) | `property` rule line 180 | Low (compiler work) |
| `import` with wildcard and module (`import java:com.foo.*@module`) | `importStatement` line 56 | Low |
| Multi-type catches (`catch(e1 | e2 ex)`) | `catches` rule line 417 | Low |
| `bx:component` tag syntax | `component` rule line 260 | Out of scope (template) |

### 3.3 Phase 3: Expression Parity

Expressions present in BoxLang but missing from MatchBox:

| Feature | BoxLang Grammar | Priority |
|---|---|---|
| Bitwise operators: `b\|`, `b&`, `b^`, `b~` | lines 668–670, 649 | High |
| Bitwise shifts: `b<<`, `b>>`, `b>>>` | lines 664–666 | High |
| Range operators: `..`, `>..`, `..<`, `>..<` | lines 657–662 | High |
| `castAs` operator | line 679 | High |
| `instanceOf` operator | line 671 | High |
| `contains` / `does not contain` operators | line 680 | High |
| Power operator `^` | line 651 | High |
| `XOR` logical operator | line 677 | High |
| `EQV` logical operator | line 674 | High |
| String concatenation `&` (currently overloaded, needs separation from bitwise `b&`) | line 672 | High |
| `::method` functional BIF reference | line 688 | High |
| Compound concat assignment `&=` | line 698 | High |
| Object destructuring `{ a, b } = obj` | line 701 | High |
| Array destructuring `[ a, b ] = arr` | line 703 | High |
| Spread expression `...expr` in args/arrays | lines 149, 293 | Medium |
| `exprStatInvocable` (complex chain as statement) | line 616 | Medium |
| `exprHeadless` (`.identifier(args?)` at expression start) | line 626 | Medium |
| `exprOutString` (`#expr#` outside strings) | line 685 | Medium |
| `var` / `final` / `static` in expression position | line 709 | Medium |
| `DOT_FLOAT_LITERAL` (`foo.50`) | line 645 | Low |
| `DOT_NUMBER_PREFIXED_IDENTIFIER` (`foo.50bar`) | line 646 | Low |
| Compound struct/array `[=]`, `[:]` | lines 437–438 | Low |
| Post-annotation on closures/lambdas | line 192 | Low |

### 3.4 Phase 4: Template Support

Full template parsing (out of scope for this PRD — will be a follow-up PRD).

---

## 4. Out of Scope

- **CFML compatibility** — BoxLang's `CFLexer.g4` / `CFGrammar.g4` / `CFTranspilerVisitor`. MatchBox targets BoxLang native syntax only.
- **SQL Query-of-Queries** — BoxLang's `SQLLexer.g4` / `SQLGrammar.g4` / `SQLVisitor`. Separate concern.
- **Doc comment parsing** — BoxLang's `DocLexer.g4` / `DocGrammar.g4`. Low priority, can be added later.
- **Template parsing beyond `.bxm`** — Full component/template grammar with modes. Follow-up PRD.
- **Compiler backend changes** — The AST node additions implied by new syntax will need compiler support, but the compiler is responsible for *behavior*, not *parsing*. Phase 2/3 items marked "compiler work" may need compiler changes beyond what a parser change alone requires.

---

## 5. Non-Goals

- Byte-for-byte identical AST to BoxLang Java AST. MatchBox has its own AST types.
- Matching BoxLang's error message text exactly. We match syntax, not error strings.
- JS/Rust/Java interop semantics — those are compiler/runtime concerns.

---

## 6. Success Criteria

1. All `.bxs` test files in `tests/scripts/` pass with the new parser producing identical AST output to the old pest parser.
2. All BoxLang-native syntax constructs listed in phases 2 and 3 parse without errors.
3. Round-trip fuzzing: parsed AST survives `Display` → re-parse → structural equality.
4. Parse error messages include source location (line:col) and are at least as good as current pest errors.
5. `matchbox-compiler` no longer depends on `pest` or `pest_derive`.
6. No performance regression vs current pest parser (target: same or faster).

---

## 7. Risks

- **Lexer mode complexity** — BoxLang's 16 lexer modes handle the mixed scripting/templating nature. If template support is deferred, only the string interpolation mode and the default script mode are needed initially. This reduces complexity significantly.
- **`&` operator ambiguity** — In BoxLang, `&` is both string concatenation and the non-bitwise `AND`. The lexer/parser must handle this correctly. `b&` is bitwise AND. We'll follow BoxLang's approach.
- **Compound assignment expansion** — `a += b` desugars to `a = a + b` in the AST. The new parser must do this correctly, matching the existing behavior.
- **Error recovery quality** — Hand-written parser error recovery is non-trivial. We'll implement simple synchronization (skip to next statement boundary) initially.

---

## 8. Implementation Plan

See individual GitHub issues for detailed breakdowns. High-level phases:

1. **P0: Custom lexer + parser foundation** — Tokenizer, Pratt parser, AST construction, drop pest.
2. **P1: Statement parity** — `do/while`, `assert`, `param`, `rethrow`, `include`, labeled breaks, modifiers.
3. **P2: Expression parity** — Bitwise, ranges, `castAs`, `instanceOf`, `contains`, destructuring, `::`, spread.
4. **P3: Full template parsing** — Separate follow-up PRD.
