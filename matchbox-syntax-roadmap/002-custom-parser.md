# 002: Implement Custom Recursive-Descent Parser

**Type:** AFK  
**Blocked by:** #001 (Custom Lexer)

## What to build

Implement a hand-written recursive-descent parser that consumes the `Vec<Token>` stream from the lexer and produces the existing AST (`Vec<Statement>`) with full fidelity to the current pest-based parser.

### Statement parser (recursive descent)

Parse all current statement forms:
- `import path.name as alias;` / `import js:path.name;`
- `class Name extends="Foo" accessors="true" implements="IBar" { members }`
- `interface Name { function_decls }`
- `function name(params) { body }` / `function name(params);` (abstract)
- `@attribute function name(params) { body }`
- `public/private String function name(params) { body }`
- `property name;`
- `for (var item, index in collection) { body }` / `for (var item in collection) body`
- `for (init; condition; update) { body }`
- `while (condition) { body }`
- `if (condition) { body } else { body }` / `if (condition) body else if (...) body`
- `switch (expr) { case val: stmts; default: stmts; }`
- `try { body } catch (ex) { body } finally { body }`
- `return expr;` / `return;`
- `throw expr;` / `throw (key=val, ...);`
- `continue;` / `break;`
- `var name = expr;`
- Expression statements

### Expression parser (Pratt parsing)

Implement Pratt parsing with correct operator precedence and associativity for all currently supported operators:

| Precedence | Operators | Associativity |
|---|---|---|
| 1 | `\|\|` | Left |
| 2 | `&&` | Left |
| 3 | `==`, `!=`, `<`, `>`, `<=`, `>=` | Left |
| 4 | `&` | Left |
| 5 | `+`, `-` | Left |
| 6 | `*`, `/`, `%` | Left |
| 7 | `!`, unary `-`, unary `+`, `++`, `--` (prefix) | Right |
| 8 | `++`, `--` (postfix), `.`, `?.`, `[]`, `()` | Postfix |

Support all current expression forms:
- Binary operations with precedence
- Unary `!`, `-`, `+`
- Prefix/postfix `++`/`--`
- Ternary `cond ? then : else` and Elvis `left ?: right`
- Assignment `target = expr` and compound `+=`, `-=`, `*=`, `/=`, `%=`
- `new path.Name(args)`
- Function calls, method calls (`.`), safe navigation (`?.`), array access (`[]`)
- Literals: numbers, booleans, `null`, strings with interpolation, arrays `[a, b]`, structs `{key: val}`, anonymous functions/closures/lambdas
- Identifiers
- Parenthesized sub-expressions

## Delivery

Parser code in `crates/matchbox-compiler/src/parser/`:
- `parse(tokens: &[Token]) -> Result<Vec<Statement>>` function
- `parse_statement()` recursive descent
- `parse_expression()` / Pratt parser core
- `parse_primary()` for atoms

Produces the **existing** AST types unchanged (`Statement`, `StatementKind`, `Expression`, `ExpressionKind`, etc.).

## Acceptance criteria

- [ ] All 86+ existing integration tests produce identical AST output compared to pest parser
- [ ] Dangling-else ambiguity handled correctly (`if x if y a else b` → else binds to inner if)
- [ ] Compound assignment desugaring works (`a += b` → `a = a + b`)
- [ ] String interpolation produces correct `StringPart` tree
- [ ] `throw(type="foo", message="bar")` desugars to struct literal
- [ ] Anonymous function + lambda parsing is correct
- [ ] Struct keys support identifiers, strings, and numbers
- [ ] Trailing commas in arrays, structs, and argument lists are accepted
- [ ] Source line information is preserved on every AST node
- [ ] Parse errors include line/column information
