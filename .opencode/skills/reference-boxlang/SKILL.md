---
name: reference-boxlang
description: Reference BoxLang JVM implementation for compatibility. Use when implementing BIFs, checking BoxLang behavior, or needing to verify how a function should work. Provides guidance on locating or cloning the BoxLang repository.
---

# Reference BoxLang Implementation

## Purpose

BoxLang has a JVM-based reference implementation that defines the canonical behavior for all BIFs, operators, and language features. When adding features to MatchBox, always check the JVM implementation first to ensure compatibility.

## Local Reference Directory

The BoxLang repository should be cloned as a subdirectory within the MatchBox project. This keeps the reference close to the code and makes paths consistent.

### Initial Setup

If the reference directory doesn't exist, clone BoxLang:

```bash
# From the matchbox project root
git clone https://github.com/ortus-boxlang/BoxLang.git reference/boxlang
```

### Keeping Up to Date

Before doing reference work, always pull the latest changes:

```bash
# From the matchbox project root
cd reference/boxlang && git pull --ff-only
```

### Gitignore

The reference directory should be excluded from version control. Add to `.gitignore` if not already present:

```
reference/
```

**Important**: The reference directory is read-only for MatchBox development. Never commit changes to it or modify BoxLang source as part of MatchBox work.

## Repository Structure

```
reference/boxlang/
├── src/main/java/ortus/boxlang/
│   ├── runtime/
│   │   ├── bifs/          # BIF implementations
│   │   │   ├── global/    # Global BIFs
│   │   │   │   ├── string/    # String BIFs (val, repeatString, etc.)
│   │   │   │   ├── math/      # Math BIFs
│   │   │   │   ├── array/     # Array BIFs
│   │   │   │   ├── struct/    # Struct BIFs
│   │   │   │   └── ...
│   │   │   └── ...
│   │   ├── types/         # Type system
│   │   ├── caster/        # Type coercion
│   │   ├── operators/     # Operator implementations
│   │   └── ...
│   └── compiler/          # Compiler/parser implementation
└── src/test/java/         # Test implementations
```

## Finding Implementations

### BIFs

```bash
# Find a specific BIF
find reference/boxlang/src/main/java -name "*Val.java"
find reference/boxlang/src/main/java -path "*/bifs/global/string/*" -name "*.java"

# Read implementation
cat reference/boxlang/src/main/java/ortus/boxlang/runtime/bifs/global/string/Val.java
```

### Parser / Syntax

```bash
# Find parser components
find reference/boxlang/src/main/java -path "*/compiler/*" -name "*.java"

# Find specific syntax handlers
grep -r "for.*loop" reference/boxlang/src/main/java/ortus/boxlang/compiler/ --include="*.java" -l
```

### Type Coercion

```bash
# Find caster implementations
find reference/boxlang/src/main/java -path "*/caster/*" -name "*.java"

# Example: Number casting
cat reference/boxlang/src/main/java/ortus/boxlang/runtime/caster/NumberCaster.java
```

### Operators

```bash
# Find operator implementations
find reference/boxlang/src/main/java -path "*/operators/*" -name "*.java"

# Example: Contains operator
cat reference/boxlang/src/main/java/ortus/boxlang/runtime/operators/Contains.java
```

## Understanding BIF Structure

Each BIF Java file follows this pattern:

```java
package ortus.boxlang.runtime.bifs.global.string;

import ortus.boxlang.runtime.bifs.BIF;
import ortus.boxlang.runtime.bifs.BoxBIF;
import ortus.boxlang.runtime.context.IBoxContext;
import ortus.boxlang.runtime.scopes.ArgumentsScope;
import ortus.boxlang.runtime.scopes.Key;
import ortus.boxlang.runtime.types.Argument;

@BoxBIF
public class Val extends BIF {
    
    public Val() {
        super();
        declaredArguments = new Argument[] {
            new Argument( true, "string", Key.string )
        };
    }
    
    public Object _invoke( IBoxContext context, ArgumentsScope arguments ) {
        var input = arguments.getAsString( Key.string );
        // Implementation logic here
        return result;
    }
}
```

Key elements:
- `@BoxBIF` annotation marks it as a BIF
- Constructor declares arguments with `declaredArguments`
- `_invoke` method contains the implementation
- Arguments accessed via `arguments.getAsString()`, `getAsInteger()`, etc.

## Member Methods (@BoxMember)

Many BIFs are also available as member methods on types (arrays, structs, strings). Look for the `@BoxMember` annotation to identify these:

```java
@BoxBIF
@BoxMember( type = MemberType.ARRAY )
public class ArrayShift extends BIF {
    // ...
}
```

**MemberType values:**
- `MemberType.ARRAY` — method on arrays (e.g., `arr.shift()`)
- `MemberType.STRUCT` — method on structs (e.g., `s.keyExists("x")`)
- `MemberType.STRING` / `MemberType.STRING_STRICT` — method on strings (e.g., `str.startsWith("x")`)

### The `objectArgument` Attribute

The `@BoxMember` annotation can specify `objectArgument` which determines which parameter the receiver maps to:

```java
@BoxBIF
@BoxMember( type = BoxLangType.STRING_STRICT, name = "Insert", objectArgument = "string" )
public class Insert extends BIF {
    public Insert() {
        super();
        declaredArguments = new Argument[] {
            new Argument( true, "string", Key.substring ),   // position 0
            new Argument( true, "string", Key.string ),      // position 1 (receiver)
            new Argument( true, "integer", Key.position )    // position 2
        };
    }
}
```

**Argument order implications:**

| Call Form | Example | args[0] | args[1] | args[2] |
|-----------|---------|---------|---------|---------|
| Function | `insert(sub, str, pos)` | substring | string | position |
| Method | `"str".insert(sub, pos)` | string (receiver) | substring | position |

When `objectArgument` is specified, the receiver becomes that argument's position, shifting other arguments. Your MatchBox implementation must handle both orderings, or the method form requires special handling in `resolve_member_method()`.

When a BIF has `@BoxMember`, you must also register it in `resolve_member_method()` in MatchBox's VM. See the add-bif skill for details.

## Reading Tests

Always read the JVM tests — they reveal expected behavior that implementation alone might not show:

```bash
# Find tests for a BIF
find reference/boxlang/src/test/java -name "*ValTest.java"

# Read the test
cat reference/boxlang/src/test/java/ortus/boxlang/runtime/bifs/global/string/ValTest.java
```

## Running BoxLang Tests

If you need to verify behavior by running JVM tests:

```bash
cd reference/boxlang

# Run specific test class
./gradlew test --tests "*ValTest*"

# Run all tests in a package
./gradlew test --tests "ortus.boxlang.runtime.bifs.global.string.*"
```

## Running BoxLang Scripts

To verify runtime behavior, you can compile and run BoxLang scripts using the reference implementation.

### Using Gradle (Recommended for Development)

Run scripts directly from the source tree without building a JAR:

```bash
# From the matchbox project root
cd reference/boxlang

# Run a script file
./gradlew run --args="script.bxs"

# Run with an absolute path
./gradlew run --args="/path/to/script.bxs"

# Run inline code
./gradlew run --args='--bx-code println("Hello")'

# Run with debug mode
./gradlew run --args="--bx-debug script.bxs"

# Pass arguments to the script
./gradlew run --args="script.bxs arg1 arg2"
```

### Building and Running the JAR

For a standalone executable:

```bash
# From reference/boxlang directory
cd reference/boxlang

# Build the shadow JAR (includes all dependencies)
./gradlew shadowJar

# Run the JAR
java -jar build/libs/boxlang-*-all.jar script.bxs

# Or with inline code
java -jar build/libs/boxlang-*-all.jar --bx-code "println('test')"
```

### Using the Installed BoxLang CLI

If you have BoxLang installed globally (via BVM or other means):

```bash
# Run a script
boxlang script.bxs

# Run inline code
boxlang --bx-code "println('test')"

# Run with debug mode
boxlang --bx-debug script.bxs
```

### Common CLI Flags

- `--bx-debug` - Enable debug mode with timing information
- `--bx-code <code>` - Execute inline BoxLang code
- `--bx-config <path>` - Use custom configuration file
- `--bx-printAST` - Print the Abstract Syntax Tree
- `--bx-transpile` - Transpile BoxLang to Java source
- `--version` - Show version information

### Example: Verifying BIF Behavior

```bash
# Create a test script
cat > /tmp/test_val.bxs << 'EOF'
var result = val( "123abc" );
println( "Result: " & result );
println( "Type: " & getType( result ) );
EOF

# Run on BoxLang JVM to see expected behavior
cd reference/boxlang
./gradlew run --args="/tmp/test_val.bxs"

# Compare with MatchBox
cd /home/jacob/dev/ortus-boxlang/matchbox
cargo run -- /tmp/test_val.bxs
```

The BoxLang JVM output is the specification. MatchBox must produce identical output.

## Workflow

When implementing or fixing a feature in MatchBox:

1. **Update reference** — `cd reference/boxlang && git pull --ff-only`
2. **Find the source** — Locate the relevant Java implementation
3. **Read the code** — Understand the logic, edge cases, return types
4. **Read the tests** — Understand expected behavior from test cases
5. **Run on JVM** — Verify actual behavior with a test script if needed
6. **Implement in MatchBox** — Port the logic to Rust
7. **Write tests** — Create integration tests that match JVM behavior
8. **Verify** — Ensure MatchBox output matches JVM output exactly

## Common Patterns

### String operations
- Null/empty checks first
- Character-by-character iteration when needed
- StringBuilder for building results
- Type casting via `NumberCaster.cast()` or similar

### Type conversions
- `NumberCaster.cast()` for numeric conversions
- `arguments.getAsString()` for string arguments
- `arguments.getAsInteger()` for integer arguments
- Most BIFs return sensible defaults rather than throwing

### Error handling
- Most BIFs return defaults (0, "", false) for invalid input
- Some throw `BoxRuntimeException` for truly invalid cases
- Check tests to see which pattern a BIF uses

## Tips

- BoxLang is case-insensitive for BIF names
- Arguments can be positional or named
- Many BIFs work as both functions and methods (e.g., `val(str)` and `str.val()`)
- Check if a BIF is registered as a string/array/struct method in the JVM implementation
- The reference directory is read-only — never modify BoxLang source for MatchBox work

## Related Skills

- `matchbox-compat` — Workflow for identifying and fixing compatibility differences
- `add-bif` — Detailed workflow for adding new BIFs to MatchBox
