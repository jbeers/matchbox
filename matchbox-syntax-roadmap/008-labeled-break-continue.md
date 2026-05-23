# 008: Add Labeled `break`/`continue`

**Type:** AFK  
**Blocked by:** #003 (Wire New Parser)

## What to build

Add labeled `break` and `continue` support through all compiler layers.

BoxLang syntax:
```
break label;
continue label;
```

Labels allow breaking/continuing out of nested loops by name. The label refers to a named enclosing loop or switch.

## Delivery

- **Parser:** Parse `break` / `continue` keyword → optional identifier (the label)
- **AST:** Change `Break` and `Continue` variants to carry `Option<String>` label
- **Compiler:** When a label is present, the compiler must track the label of each enclosing loop. On `break label`, emit a jump to the post-loop position of the labeled loop. On `continue label`, emit a jump to the loop condition/update of the labeled loop.
- **Test:** Integration test with labeled break/continue across nested loops.

## Acceptance criteria

- [ ] `break label;` exits the labeled enclosing loop
- [ ] `continue label;` continues the labeled enclosing loop
- [ ] Unlabeled `break;` and `continue;` still work as before
- [ ] Compile error for label that doesn't match any enclosing loop
- [ ] Integration test passes
