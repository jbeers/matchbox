# 006: Add `param` Statement

**Type:** AFK  
**Blocked by:** #003 (Wire New Parser)

## What to build

Add `param` statement support through all compiler layers.

BoxLang syntax:
```
param String foo.bar = "default";
param foo.bar = "default";
param String foo.bar;
```

The `param` statement ensures a variable (or nested property) exists, setting a default if it doesn't. It's essentially "declare with default if not already defined."

## Delivery

- **Parser:** Parse `param` keyword → optional type → expression (which is an assignment or identifier chain)
- **AST:** Add `StatementKind::Param { type_name: Option<String>, target: Expression, default: Option<Expression> }` variant
- **Compiler:** Emit bytecode that checks if the target variable exists/is not null. If it doesn't exist or is null, assign the default value. If no default is provided and variable doesn't exist, throw an error.
- **Test:** Integration test covering: setting default when undefined, skipping when already defined, error on missing required param.

## Acceptance criteria

- [ ] `param name = "default";` sets default when variable is undefined
- [ ] `param name = "default";` does not overwrite existing value
- [ ] `param name;` throws error if variable is undefined (no default)
- [ ] `param Type name = value;` parses optional type annotation
- [ ] Integration test passes
