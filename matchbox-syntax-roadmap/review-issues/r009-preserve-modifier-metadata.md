# R009: Preserve Modifier Metadata For Enforcement

**Type:** Correctness / Code quality  
**Priority:** Medium  
**Related issue:** `c010-access-modifier-enforcement.md`
**Related files:** `crates/matchbox-compiler/src/ast/mod.rs`, `crates/matchbox-compiler/src/parser/mod.rs`, `crates/matchbox-compiler/src/compiler/mod.rs`

## Problem

The parser accepts modifiers such as `static`, `abstract`, and `final`, but function parsing discards them. Class-level `abstract` and `final` are not represented in `StatementKind::ClassDecl` either.

That makes C010 harder to implement correctly because the compiler cannot reliably know:

- Which classes are abstract or final.
- Which methods are static, abstract, or final.
- Whether `this` is being used from a static context.

## Solution

Store modifier metadata in the AST.

Suggested AST changes:

```rust
pub struct FunctionModifiers {
    pub access: Option<String>,
    pub is_static: bool,
    pub is_abstract: bool,
    pub is_final: bool,
}

pub struct ClassModifiers {
    pub is_abstract: bool,
    pub is_final: bool,
}
```

Then update compiler validation and class metadata to use structured modifiers instead of reparsing source text or relying on discarded tokens.

## Acceptance Criteria

- [ ] Class AST preserves `abstract` and `final`.
- [ ] Function AST preserves `static`, `abstract`, and `final`.
- [ ] Existing access modifier behavior still compiles.
- [ ] C010 enforcement can be implemented without reparsing source text.
- [ ] Tests assert modifier metadata in parser unit tests.

