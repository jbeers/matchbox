---
name: matchbox-compat
description: Identify and fix compatibility differences between MatchBox and BoxLang JVM. Use when MatchBox doesn't match BoxLang's syntax or runtime behavior — parser differences, BIF behavior, operator semantics, type coercion, error messages, or any divergence from the reference implementation.
---

# MatchBox Compatibility — Making MatchBox Match BoxLang

## Purpose

BoxLang JVM is the reference implementation. When MatchBox diverges — in syntax, runtime behavior, BIF semantics, error handling, or type coercion — MatchBox is wrong. This skill guides you through identifying, verifying, and fixing those divergences using TDD.

## Core Principle

**MatchBox must match BoxLang.** If BoxLang accepts syntax that MatchBox rejects, MatchBox's parser needs to change. If a BIF returns a different value, MatchBox's implementation needs to change. Never work around a difference in user code — fix it in MatchBox.

## Workflow

### Step 1: Identify the Difference

Differences surface in several ways:

- **User reports**: A BoxLang script compiles/runs on JVM but fails in MatchBox
- **Test failures**: BoxLang language tests (`~/dev/ortus-boxlang/boxlang-language-tests/`) fail in MatchBox
- **Code review**: Reading BoxLang source reveals behavior MatchBox doesn't implement
- **Intuition**: "Does MatchBox handle X the same way BoxLang does?"

Create a minimal reproduction — the smallest BoxLang script that demonstrates the divergence.

### Step 2: Verify BoxLang's Official Behavior

Before changing anything, confirm what BoxLang JVM actually does. **Never assume** — always run the script.

```bash
# Write a minimal test script
cat > /tmp/compat_test.bxs << 'EOF'
// Test the specific behavior in question
var result = <the thing you're testing>;
println( "Result: " & result );
println( "Type: " & getType( result ) );
EOF

# Run on BoxLang JVM
boxlang /tmp/compat_test.bxs

# Run on MatchBox for comparison
cargo run --manifest-path ~/dev/ortus-boxlang/matchbox/Cargo.toml -- /tmp/compat_test.bxs
```

Capture both outputs. The BoxLang JVM output is the spec.

**Test edge cases on JVM too** — null inputs, empty strings, wrong types, boundary values. BoxLang's behavior on these IS the spec.

### Step 3: Study the BoxLang Implementation

Understand *how* BoxLang achieves this behavior before porting it.

```bash
# Find the relevant source
# For BIFs:
find ~/dev/ortus-boxlang/BoxLang/src/main/java -name "*.java" | xargs grep -l "FunctionName"

# For parser/syntax features:
find ~/dev/ortus-boxlang/BoxLang/src/main/java -path "*/compiler/*" -name "*.java"

# For type coercion:
find ~/dev/ortus-boxlang/BoxLang/src/main/java -path "*/caster*" -name "*.java"

# For operator behavior:
find ~/dev/ortus-boxlang/BoxLang/src/main/java -path "*/operator*" -name "*.java"
```

Read the implementation AND its tests. The JVM tests reveal expected behavior that the implementation alone might not.

```bash
# Find and read JVM tests
find ~/dev/ortus-boxlang/BoxLang/src/test/java -name "*Test.java" | xargs grep -l "FunctionName"
```

See the `reference-boxlang` skill for detailed repo navigation guidance.

### Step 4: Fix MatchBox Using Red-Green TDD

This is where you make the change. Load the `tdd` skill for full TDD methodology. The pattern:

#### RED — Write a failing test

Create a test script in `tests/scripts/` that exercises the behavior:

```bash
# tests/scripts/vm_<feature_name>.bxs
// This should match BoxLang JVM behavior exactly
var result = <the thing that should work>;
if ( result != <expected value from JVM> ) {
    throw "Expected <expected>, got " & result;
}
println( "OK" );
```

Register it in `tests/integration_tests.rs`:

```rust
script_test!(vm_<feature_name>, "vm_<feature_name>.bxs");
```

Run it and confirm it **fails**:

```bash
cargo test vm_<feature_name>
```

#### GREEN — Make it pass

Implement the minimum change to make the test pass. This could be:

- **Parser change**: `crates/matchbox-compiler/` — for syntax differences
- **BIF implementation**: `crates/matchbox-vm/src/bifs/` — for function behavior
- **Type coercion**: `crates/matchbox-vm/src/casters/` — for type conversion differences
- **Operator semantics**: `crates/matchbox-vm/src/operators/` — for operator behavior
- **Runtime**: `crates/matchbox-vm/src/vm/` — for execution model differences

Run the test and confirm it **passes**:

```bash
cargo test vm_<feature_name>
```

#### Verify no regressions

```bash
cargo test
```

#### Refactor

Clean up the implementation while keeping tests green. Match existing code patterns and style in the codebase.

### Step 5: Cross-Check Against JVM

After your fix passes, run the original reproduction script on both runtimes to confirm identical output:

```bash
boxlang /tmp/compat_test.bxs
cargo run --manifest-path ~/dev/ortus-boxlang/matchbox/Cargo.toml -- /tmp/compat_test.bxs
```

If outputs match, the compatibility issue is resolved.

## Categories of Differences

### Parser / Syntax

MatchBox's parser (`crates/matchbox-compiler/`) must accept everything BoxLang's parser accepts. Examples:
- Loop syntax (`for ( var x in y )` vs `for ( x in y )`)
- Catch block syntax (`catch ( any e )` vs `catch ( e )`)
- Operator syntax (`.contains()` method vs `contains` operator)
- String literal escaping

**Process**: Run the syntax on JVM → confirm it compiles → add parser support in MatchBox.

### BIF Behavior

Functions must return identical results for identical inputs. This includes:
- Return types (number vs string vs boolean)
- Edge cases (null, empty, missing arguments)
- Error messages and exception types
- Method vs function syntax support

**Process**: Run on JVM with edge cases → document exact behavior → port to MatchBox.
See the `add-bif` skill for detailed BIF implementation guidance.

### Type Coercion

BoxLang has specific rules for implicit type conversion. MatchBox must match these:
- String-to-number coercion in comparisons
- Truthy/falsy evaluation
- Array/struct/string interop
- Null handling

**Process**: Test coercion on JVM → document rules → implement in MatchBox casters.

### Operator Semantics

Operators like `==`, `contains`, `&`, `+=` must behave identically:
- Operand types accepted
- Return types
- Short-circuit evaluation
- Associativity

**Process**: Test operator on JVM with various operand types → match in MatchBox.

### Error Handling

Error messages and exception behavior should match:
- What errors are thrown (vs returned)
- Error message format
- Stack trace behavior
- Try/catch semantics

## Tools and Environment

```bash
# BoxLang JVM (the spec)
boxlang script.bxs                    # Run a script
boxlang -e "println( 42 )"           # Run inline code

# MatchBox (what we're fixing)
cargo test                            # Run all tests
cargo test vm_<name>                  # Run specific test
cargo build --release                 # Build release binary
cargo run -- script.bxs              # Run script via cargo

# BoxLang JVM source (the reference)
~/dev/ortus-boxlang/BoxLang/          # Java implementation
~/dev/ortus-boxlang/boxlang-language-tests/  # Official test suite
```

## Checklist

For each compatibility fix:

- [ ] Identified the specific divergence
- [ ] Created minimal reproduction script
- [ ] Verified BoxLang JVM behavior (ran the script, captured output)
- [ ] Tested edge cases on JVM
- [ ] Read relevant BoxLang JVM source code
- [ ] Read relevant BoxLang JVM tests
- [ ] Wrote failing MatchBox test (RED)
- [ ] Implemented fix (GREEN)
- [ ] All MatchBox tests pass (no regressions)
- [ ] Cross-checked output matches JVM exactly
- [ ] Refactored if needed (tests still green)

## Related Skills

- `reference-boxlang` — Locating and navigating BoxLang JVM source
- `add-bif` — Detailed workflow for adding BIFs specifically
- `tdd` — General TDD methodology and principles
