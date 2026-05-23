# 001: Implement Custom Lexer (Tokenizer)

**Type:** AFK  
**Blocked by:** None — can start immediately

## What to build

Implement a hand-written lexer (tokenizer) for BoxLang source code. The lexer reads source text character-by-character and produces a `Vec<Token>` stream, where each `Token` carries a `TokenKind`, source span (line/col offsets), and the lexeme text.

The lexer must handle:
- All BoxLang keywords: `import`, `class`, `interface`, `property`, `function`, `return`, `var`, `required`, `for`, `while`, `in`, `if`, `else`, `try`, `catch`, `finally`, `continue`, `break`, `switch`, `case`, `default`, `throw`, `new`, `true`, `false`, `null`, `as`, `public`, `private`, `extends`, `implements`, `accessors`, `abstract`, `final`, `static`, `do`, `assert`, `param`, `rethrow`, `include`, `not`, `package`, `remote`
- All operators: arithmetic (`+`, `-`, `*`, `/`, `%`, `^`), comparison (`==`, `!=`, `<`, `>`, `<=`, `>=`), logical (`&&`, `||`, `!`), bitwise (`b|`, `b&`, `b^`, `b~`, `b<<`, `b>>`, `b>>>`), string concat (`&`), assignment (`=`, `+=`, `-=`, `*=`, `/=`, `%=`, `&=`), postfix/prefix (`++`, `--`), ternary (`?`, `:`), Elvis (`?:`), arrow/lambda (`=>`, `->`), range (`..`, `>..`, `..<`, `>..<`), `::`, `?.`, `instanceOf`, `castAs`, `contains`, `doesNotContain`, `xor`, `eqv`
- Punctuation: `{`, `}`, `(`, `)`, `[`, `]`, `,`, `.`, `;`, `:`, `@`
- Literals: integers (with `_` separators), decimals, double-quoted strings with `#expr#` interpolation and `##`/`""` escaping, single-quoted strings with `#expr#` interpolation and `''` escaping, booleans, `null`
- Identifiers: `[_$a-zA-Z][_$a-zA-Z0-9]*`
- Comments: `//` line comments, `/* */` block comments
- Whitespace: spaces, tabs, newlines (skipped)

String interpolation requires a **lexer mode** (state): when inside a double-quoted or single-quoted string, `#` followed by `#` is an escaped hash, but `#` followed by non-`#` enters expression mode where the lexer tokenizes a nested expression until the closing `#`.

## Delivery

A new `tokenizer` module in `crates/matchbox-compiler/src/` containing:
- `TokenKind` enum with all token variants
- `Token` struct with `kind`, `span`, `lexeme`
- `Span` struct with `start: usize, end: usize, line: u32, col: u32`
- `tokenize(source: &str) -> Result<Vec<Token>>` function
- `LexerMode` enum: `Default`, `StringInterpolation`

## Acceptance criteria

- [ ] All current BoxLang keywords tokenize correctly
- [ ] All operators tokenize correctly, including multi-char operators (`b<<`, `>..<`, `?:`, etc.)
- [ ] Integer literals with and without `_` separators parse correctly
- [ ] Decimal literals parse correctly
- [ ] Double-quoted strings parse correctly with `#expr#` interpolation, `##` escaping, `""` escaping
- [ ] Single-quoted strings parse correctly with `#expr#` interpolation, `''` escaping
- [ ] Line comments (`//`) and block comments (`/* */`) are skipped
- [ ] Whitespace is skipped
- [ ] Snapshot tests exist covering a representative sample of BoxLang source
- [ ] Token span information is accurate (line/col correct)
- [ ] No external dependencies added (no `logos`, no `nom`, etc.)
