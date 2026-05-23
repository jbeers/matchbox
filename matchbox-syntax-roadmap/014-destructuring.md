# 014: Add Destructuring Assignment

**Type:** AFK  
**Blocked by:** #003 (Wire New Parser)

## What to build

Add object and array destructuring assignment through all compiler layers.

BoxLang syntax:
```
// Object destructuring
{ a, b } = myStruct;
{ a: localA, b: localB } = myStruct;   // with renaming
{ a, ...rest } = myStruct;             // with rest/spread

// Array destructuring
[ a, b ] = myArray;
[ a, b, ...rest ] = myArray;           // with rest/spread
```

Destructuring extracts values from structs/arrays and binds them to local variables in a single statement.

## Delivery

- **Parser:** In assignment parsing, when `{` or `[` is encountered and the pattern contains bare identifiers (not `key: value` struct syntax), treat it as a destructuring pattern. Parse identifiers with optional rename (`localName: sourceKey` or `sourceKey: localName`) and optional rest (`...identifier`).
- **AST:** Add `ExpressionKind::ObjectDestructure { bindings, rest }` and `ExpressionKind::ArrayDestructure { bindings, rest }` variants. A binding is `(source, localName)` — source defaults to localName if no rename.
- **Compiler:** Generate bytecode that iterates the source value (struct keys or array indices), extracting each requested value and binding it to the local variable. For rest, collect remaining keys/indices into a new struct/array.
- **Test:** Integration test with object destructuring (basic, rename, rest), array destructuring (basic, rest), nested destructuring, and edge cases (empty source, missing keys).

## Acceptance criteria

- [ ] `{ a, b } = struct` destructures named keys to local variables
- [ ] `{ a: localA, b: localB } = struct` supports renaming
- [ ] `{ a, ...rest } = struct` collects remaining keys
- [ ] `[ a, b ] = array` destructures by index
- [ ] `[ a, b, ...rest ] = array` collects remaining elements
- [ ] Compile error for destructuring non-struct/non-array values
- [ ] Integration test passes
