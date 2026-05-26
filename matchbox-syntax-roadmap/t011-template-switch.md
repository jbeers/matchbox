# T011: Template switch/case/defaultcase

**Type:** AFK  
**Blocked by:** T005 (Template parser foundation)

## What to build

Parse `<bx:switch>`, `<bx:case>`, and `<bx:defaultcase>` tags.

### Syntax
```
<bx:switch expression="#expr#">
    <bx:case value="1">
        body
    </bx:case>
    <bx:case value="2">
        body
    </bx:case>
    <bx:defaultcase>
        body
    </bx:defaultcase>
</bx:switch>
```

### Key difference from script switch
In BoxLang template switches, cases do NOT fall through. Each `<bx:case>` body is self-contained — execution does not continue to the next case. This is different from script `switch` where cases fall through without `break`.

### Parsing
- `<bx:switch>`: expression attribute parsed as script expression
- `<bx:case value="...">`: value attribute parsed, body is template statements
- `<bx:defaultcase>`: default branch, body is template statements
- Map to `StatementKind::Switch { value, cases, default_case }`

## Delivery

Changes: `crates/matchbox-compiler/src/parser/template.rs`

## Acceptance criteria

- [ ] `<bx:switch expression="#x#">` parses switch value
- [ ] `<bx:case value="1">` parses case with value
- [ ] `<bx:defaultcase>` parses default branch
- [ ] Multiple cases in one switch parse correctly
- [ ] Integration test: `.bxm` with switch/case/default
