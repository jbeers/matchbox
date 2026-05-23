# C010: Access Modifier Enforcement

**Type:** AFK — Medium  
**Blocked by:** None

## Problem

The parser accepts `static`, `abstract`, `final`, `remote`, `package` modifiers on functions and classes, and the AST stores them. But the compiler doesn't enforce any of their semantics.

## Solution

Add compile-time validation passes in the compiler.

### abstract enforcement
- Abstract functions: already skipped during compilation (no body to emit)
- Abstract classes: track in a set. When `new AbstractClass()` is compiled, check and error.
- Missing abstract method implementation: when a concrete class extends an abstract class, verify all abstract methods are implemented.

### final enforcement
- Final classes: track in a set. When `class Child extends FinalParent` is compiled, check and error.
- Final methods: when a method override is detected, check if parent method is final.

### static enforcement
- Static functions: should not have access to `this` or `super`. Check and error.
- Static dispatch: compile static function calls without instance context.

### remote / package
- Metadata only for now. Future: remote functions are HTTP-accessible, package functions are package-scoped.

### Implementation approach
```rust
struct ClassRegistry {
    abstract_classes: HashSet<String>,
    final_classes: HashSet<String>,
}
```

During compilation:
1. First pass: collect all class declarations and their modifiers
2. Second pass: validate instantiation, inheritance, and overrides
3. Report errors at compile time

### Test
```
abstract class Animal {
    abstract function speak();
}
class Dog extends Animal {
    function speak() { return "Woof"; }
}
// Should compile: Dog implements speak()

// Should error:
// var a = new Animal();              // Cannot instantiate abstract class
// class Cat extends Animal { }       // Missing abstract method speak()

final class Sealed { }
// class Child extends Sealed { }    // Cannot extend final class
```

## Acceptance criteria
- [ ] Abstract class instantiation produces compile error
- [ ] Missing abstract method implementation produces compile error
- [ ] Extending final class produces compile error
- [ ] Overriding final method produces compile error
- [ ] Using `this` in static context produces compile error
- [ ] Integration tests for each error case
