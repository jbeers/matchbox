# T012: Template include + import + throw/rethrow

**Type:** AFK  
**Blocked by:** T005 (Template parser foundation)

## What to build

Parse `<bx:include>`, `<bx:import>`, `<bx:throw>`, `<bx:rethrow>`, and `<bx:property>` tags.

### Syntax
```
<bx:include template="path/to/file.bxm">
<bx:include template="path/to/file.bxm" />
<bx:import prefix="java" name="java.util.ArrayList" [alias="List"]>
<bx:throw message="Error occurred" [detail="..." type="..." ...]>
<bx:rethrow>
<bx:rethrow />
<bx:property name="foo" type="string" default="bar">
```

### Parsing
- `<bx:include>`: Parse `template` attribute, produce `StatementKind::Include`
- `<bx:import>`: Parse `prefix`, `name`, optional `alias` attributes, produce `StatementKind::Import`
- `<bx:throw>`: Parse attributes as struct (message, detail, type, etc.), produce `StatementKind::Throw`
- `<bx:rethrow>`: Produce `StatementKind::Rethrow`
- `<bx:property>`: Parse attributes, emit as property declaration

## Delivery

Changes: `crates/matchbox-compiler/src/parser/template.rs`

## Acceptance criteria

- [ ] `<bx:include template="...">` parses correctly
- [ ] `<bx:import>` with prefix/name/alias parses
- [ ] `<bx:throw message="...">` parses structured exception
- [ ] `<bx:rethrow>` and self-closing form parse
- [ ] `<bx:property>` parses with attributes
- [ ] Integration tests for each tag
