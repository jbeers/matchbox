# 018: Add Access Modifiers and Modifiers

**Type:** AFK  
**Blocked by:** #003 (Wire New Parser)

## What to build

Add all BoxLang access modifiers and class/function modifiers through all compiler layers.

BoxLang modifiers:
```
// Access modifiers (currently only public/private are supported)
public    — accessible everywhere
private   — accessible within same class
remote    — accessible via remote invocation (HTTP, etc.)
package   — accessible within same package

// Other modifiers (not supported at all currently)
static    — static member/method
abstract  — abstract class/method (no body)
final     — cannot be overridden/subclassed
default   — default implementation in interface
```

## Delivery

- **Parser:** 
  - Extend function declaration parsing to accept `remote` and `package` alongside existing `public`/`private`
  - Add `static`, `abstract`, `final` keyword parsing before `function` keyword and `class` keyword
  - Add `default` keyword before `function` in interface declarations (for default method implementations)
- **AST:** 
  - Extend `access_modifier` field to support `"public"`, `"private"`, `"remote"`, `"package"`
  - Add `modifiers: Vec<String>` field to `FunctionDecl` and `ClassDecl` for `static`, `abstract`, `final`
  - Add `is_default: bool` to `FunctionDecl` for interface default methods
- **Compiler:** 
  - `remote`: allow invocation metadata generation
  - `package`: scope-check at compile/runtime
  - `static`: emit static initializer, resolve `this`/`super` restrictions
  - `abstract`: validate no body, enforce subclass implementation
  - `final`: prevent override at compile time
  - `default` (interface): allow method body in interface
- **Test:** Integration tests for each modifier combination.

## Acceptance criteria

- [ ] `remote function foo() {}` parses and compiles
- [ ] `package function foo() {}` parses and compiles
- [ ] `static function foo() {}` parses and compiles (static dispatch)
- [ ] `abstract function foo();` parses — no body required
- [ ] `final function foo() {}` parses — cannot be overridden
- [ ] `final class Foo {}` parses — cannot be extended
- [ ] `abstract class Foo {}` parses — cannot be instantiated directly
- [ ] `default function foo() {}` in interface parses — provides default impl
- [ ] Modifiers can be combined: `public static function`, `private final function`, etc.
- [ ] Integration test passes
