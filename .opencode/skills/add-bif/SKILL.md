---
name: add-bif
description: Add a new built-in function (BIF) to MatchBox using TDD. Use when implementing new BIFs, adding string/math/array/struct functions, or extending MatchBox's standard library. Follows red-green-refactor pattern with BoxLang JVM reference.
---

# Add BIF to MatchBox

## Overview

This skill guides you through adding a new built-in function (BIF) to MatchBox using test-driven development. Always reference the BoxLang JVM implementation for compatibility.

## Two Implementation Approaches

MatchBox supports two ways to implement BIFs:

### 1. Native Rust BIFs (in `crates/matchbox-vm/src/bifs/`)

Use this approach when:
- The BIF requires direct access to VM internals (array manipulation, GC objects)
- The BIF involves system operations (file I/O, HTTP, crypto)
- Performance is critical
- The BIF cannot be expressed in BoxLang itself

### 2. Prelude BIFs (in `crates/matchbox-compiler/src/prelude.bxs`)

Use this approach when:
- The BIF can be composed from existing primitives
- The BIF is a higher-order function (map, filter, reduce, each, etc.)
- The implementation is simpler in BoxLang than Rust
- The BIF iterates over collections using existing functions

The prelude is written in BoxLang and gets tree-shaken during compilation (only used BIFs are included in output).

### Decision Tree

```
Can it be implemented using existing MatchBox primitives?
├── YES → Implement in prelude.bxs
│   Examples: arrayMap, arrayFilter, structEach, arrayToList
│
└── NO → Implement as native Rust BIF
    Examples: fileRead, http, hash, queryExecute
```

## Prerequisites

Before starting, ensure you have:
1. MatchBox repository cloned locally
2. Rust toolchain installed
3. BoxLang JVM repository available (see `reference-boxlang` skill)

## Workflow

### Step 1: Research the BIF in BoxLang JVM

First, understand how the BIF works in the reference implementation:

```bash
# Use the reference-boxlang skill to locate BoxLang
# Then find the BIF implementation
find ~/dev/ortus-boxlang/BoxLang/src/main/java/ortus/boxlang/runtime/bifs -name "*<BifName>*.java"

# Read the implementation
cat <path-to-bif>.java

# Check the tests
find ~/dev/ortus-boxlang/BoxLang/src/test/java -name "*<BifName>*Test.java"
cat <path-to-test>.java
```

Key things to note:
- What arguments does it take?
- What does it return?
- How does it handle edge cases (null, empty, invalid input)?
- Is it also a string/array/struct method?

### Step 2: Write Test (RED Phase)

Create a test script in `tests/scripts/`:

```bash
# Create test file
cat > tests/scripts/vm_<bif_name>.bxs << 'EOF'
// Test <bif_name> function
var result1 = <bif_name>( "arg1", "arg2" );
if ( result1 != expected1 ) { throw "<bif_name> basic test failed: got " & result1; }

var result2 = <bif_name>( "edge_case" );
if ( result2 != expected2 ) { throw "<bif_name> edge case failed: got " & result2; }

// Test as method if applicable
var str = "test";
if ( !str.<bif_name>( "arg" ) ) { throw "<bif_name> method failed"; }

println( "<bif_name> OK" );
EOF
```

Register the test in `tests/integration_tests.rs`:

```rust
script_test!(vm_<bif_name>, "vm_<bif_name>.bxs");
```

Run the test to confirm it fails:

```bash
cargo test vm_<bif_name>
```

Expected: Test fails with "Can only call functions" or similar error.

### Step 3: Implement BIF (GREEN Phase)

Add the BIF implementation in `crates/matchbox-vm/src/bifs/mod.rs`:

```rust
fn <bif_name>_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() != <expected_arg_count> {
        return Err("<bif_name>() expects exactly <N> arguments".to_string());
    }
    
    let input = vm.to_string(args[0]);
    // Implementation logic here (port from BoxLang JVM)
    
    Ok(BxValue::new_string(result))
}
```

Register the BIF in the `register_all()` function:

```rust
bifs.insert("<bif_name>".to_string(), <bif_name>_bif as BxNativeFunction);
```

If the BIF is also a string method, register it in `crates/matchbox-vm/src/vm/mod.rs`:

```rust
// In resolve_member_method(), under GcObject::String(_) match arm:
"<bif_name>" => Some("<bif_name>".to_string()),
```

### Step 3b: Implement BIF in Prelude (Alternative to Rust)

If the BIF can be implemented using existing primitives, add it to `crates/matchbox-compiler/src/prelude.bxs`:

```boxlang
/**
 * Returns true if any element in the array satisfies the predicate.
 */
function arraySome(array, predicate) {
    for (item in array) {
        if (predicate(item)) {
            return true;
        }
    }
    return false;
}

/**
 * Returns true if no elements in the array satisfy the predicate.
 */
function arrayNone(array, predicate) {
    return !arraySome(array, predicate);
}
```

**Prelude BIF guidelines:**
- Include a JSDoc-style comment describing the function
- Use existing BIFs and language constructs
- Follow the naming conventions (camelCase for function names)
- The function will be available globally (no module exports needed)
- Tree-shaking ensures unused prelude BIFs don't bloat output

**When to use prelude vs. Rust:**

| Prelude (BoxLang) | Native (Rust) |
|-------------------|---------------|
| `arrayMap`, `arrayFilter`, `arrayReduce` | `arrayAppend`, `arrayDeleteAt` |
| `structEach`, `structMap`, `structFilter` | `structKeyExists`, `structInsert` |
| `arrayToList`, `arrayReverse`, `arraySlice` | `len`, `toString`, `duplicate` |
| Composed from existing primitives | Direct VM/system access |

Run the test again:

```bash
cargo test vm_<bif_name>
```

Expected: Test passes.

### Step 4: Run Full Test Suite

Ensure no regressions:

```bash
cargo test
```

All tests should pass.

### Step 5: Build and Test Locally

Build MatchBox with the new BIF:

```bash
cargo build --release --features "bif-http,bif-zip"
```

Test with a real BoxLang script:

```bash
cat > test_new_bif.bxs << 'EOF'
var result = <bif_name>( "test" );
println( "Result: " & result );
EOF

./target/release/matchbox test_new_bif.bxs
```

### Step 6: Commit

```bash
git add -A
git commit -m "feat: add <bif_name> BIF

Implements <bif_name> function matching BoxLang JVM behavior.
- Handles <edge_case_1>
- Handles <edge_case_2>
- Works as both function and method

Closes #<issue_number>"
```

## Implementation Patterns

### String BIFs

```rust
fn string_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 1 {
        return Err("stringBif() expects at least 1 argument".to_string());
    }
    
    let input = vm.to_string(args[0]);
    let result = input.chars().filter(|c| c.is_alphanumeric()).collect::<String>();
    
    Ok(BxValue::new_string(result))
}
```

### Numeric BIFs

```rust
fn numeric_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() != 1 {
        return Err("numericBif() expects exactly 1 argument".to_string());
    }
    
    let num = args[0].as_number();
    let result = num * 2.0;
    
    Ok(BxValue::new_number(result))
}
```

### Boolean BIFs

```rust
fn boolean_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() != 2 {
        return Err("booleanBif() expects exactly 2 arguments".to_string());
    }
    
    let input = vm.to_string(args[0]);
    let pattern = vm.to_string(args[1]);
    let result = input.contains(&pattern);
    
    Ok(BxValue::new_bool(result))
}
```

### Array BIFs

```rust
fn array_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() != 1 {
        return Err("arrayBif() expects exactly 1 argument".to_string());
    }
    
    let array_id = args[0].as_gc_id()
        .ok_or("arrayBif() expects an array argument")?;
    
    let len = vm.array_len(array_id);
    let mut result_vec = Vec::new();
    
    for i in 0..len {
        let item = vm.array_get(array_id, i);
        // Process item
        result_vec.push(item);
    }
    
    let result_id = vm.array_new();
    for item in result_vec {
        vm.array_push(result_id, item);
    }
    
    Ok(BxValue::new_ptr(result_id))
}
```

## Common Edge Cases

### Null/Empty Handling

```rust
// BoxLang typically returns sensible defaults
if input.is_empty() {
    return Ok(BxValue::new_string(""));  // or 0, or false
}
```

### Type Coercion

```rust
// BoxLang is loosely typed - handle various input types
let num = match &args[0] {
    BxValue::Number(n) => *n,
    BxValue::String(s) => vm.to_string(args[0]).parse().unwrap_or(0.0),
    _ => 0.0,
};
```

### Case Insensitivity

```rust
// BoxLang BIF names are case-insensitive
// Register with lowercase name
bifs.insert("mybif".to_string(), my_bif as BxNativeFunction);

// Method names are also case-insensitive (handled by resolve_member_method)
```

## Testing Patterns

### Basic Functionality

```boxlang
var result = myBif( "input" );
if ( result != "expected" ) { throw "basic test failed"; }
```

### Edge Cases

```boxlang
// Empty input
var empty = myBif( "" );
if ( empty != "" ) { throw "empty input failed"; }

// Null handling (if applicable)
var nullResult = myBif( javaCast( "null", "" ) );
```

### Method Syntax

```boxlang
var str = "test";
if ( !str.myBif( "arg" ) ) { throw "method syntax failed"; }
```

### Multiple Arguments

```boxlang
var result = myBif( "arg1", "arg2", "arg3" );
if ( result != "expected" ) { throw "multi-arg failed"; }
```

## Checklist

Before submitting:

- [ ] Researched BoxLang JVM implementation
- [ ] Decided: prelude (BoxLang) vs native (Rust) implementation
- [ ] Wrote comprehensive tests (basic, edge cases, method syntax)
- [ ] Test fails before implementation (RED)
- [ ] Implementation matches BoxLang behavior
- [ ] Test passes after implementation (GREEN)
- [ ] Full test suite passes (no regressions)
- [ ] Built and tested locally with real script
- [ ] Committed with descriptive message

### Prelude-specific checklist:

- [ ] Can be implemented using existing primitives (no VM internals needed)
- [ ] Includes JSDoc-style documentation comment
- [ ] Uses camelCase naming convention
- [ ] Tested with tree-shaking enabled (default build)

## Examples from Recent Work

### arrayMap (Prelude BIF)

```bash
# 1. Research in BoxLang
cat ~/dev/ortus-boxlang/BoxLang/src/main/java/ortus/boxlang/runtime/bifs/global/array/ArrayMap.java

# 2. Write test
cat > tests/scripts/vm_array_map.bxs << 'EOF'
var arr = [1, 2, 3];
var doubled = arrayMap(arr, (x) => x * 2);
if (doubled[1] != 2 || doubled[2] != 4 || doubled[3] != 6) {
    throw "arrayMap failed";
}
println("arrayMap OK");
EOF

# 3. Implement in prelude.bxs (NOT in Rust)
# Add to crates/matchbox-compiler/src/prelude.bxs:

/**
 * Maps an array to a new array using a callback function.
 */
function arrayMap(array, callback) {
    var result = [];
    for (item in array) {
        arrayAppend(result, callback(item));
    }
    return result;
}

# 4. Test - no Rust compilation needed!
cargo test vm_array_map
```

### stringEndsWith (Native Rust BIF)

```bash
# 1. Research in BoxLang
cat ~/dev/ortus-boxlang/BoxLang/src/main/java/ortus/boxlang/runtime/bifs/global/string/StringEndsWith.java

# 2. Write test
cat > tests/scripts/vm_string_ends_with.bxs << 'EOF'
var result = stringEndsWith( "hello world", "world" );
if ( result != true ) { throw "stringEndsWith failed"; }
var str = "test.txt";
if ( !str.endsWith( ".txt" ) ) { throw "method failed"; }
println( "stringEndsWith OK" );
EOF

# 3. Implement
# In crates/matchbox-vm/src/bifs/mod.rs:
fn string_ends_with_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    let input = vm.to_string(args[0]);
    let suffix = vm.to_string(args[1]);
    Ok(BxValue::new_bool(input.ends_with(&suffix)))
}

# Register:
bifs.insert("stringendswith".to_string(), string_ends_with_bif as BxNativeFunction);

# In crates/matchbox-vm/src/vm/mod.rs, add to string methods:
"endswith" => Some("stringendswith".to_string()),
```

### val

```bash
# 1. Research in BoxLang
cat ~/dev/ortus-boxlang/BoxLang/src/main/java/ortus/boxlang/runtime/bifs/global/string/Val.java

# 2. Write test with edge cases
cat > tests/scripts/vm_val.bxs << 'EOF'
var result1 = val( "123abc" );
if ( result1 != 123 ) { throw "val basic failed"; }
var result2 = val( "abc" );
if ( result2 != 0 ) { throw "val no digits failed"; }
var result3 = val( "45.67xyz" );
if ( result3 != 45.67 ) { throw "val decimal failed"; }
println( "val OK" );
EOF

# 3. Implement
fn val_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    let input = vm.to_string(args[0]);
    let mut result = String::new();
    let mut found_dot = false;
    
    for c in input.chars() {
        if c.is_ascii_digit() {
            result.push(c);
        } else if c == '.' && !found_dot {
            found_dot = true;
            result.push(c);
        } else {
            break;
        }
    }
    
    let num: f64 = result.parse().unwrap_or(0.0);
    Ok(BxValue::new_number(num))
}
```

## Troubleshooting

### Prelude BIF not found

- Check spelling (must match exactly, camelCase)
- Ensure function is at top level (not nested inside another function)
- Verify `crates/matchbox-compiler/src/prelude.bxs` syntax is valid BoxLang
- Run `cargo build` to recompile the prelude

### Prelude BIF calls native BIF that doesn't exist

- Prelude can only use BIFs that are already implemented in Rust
- Check that all dependencies are registered in `register_all()`
- Example: `arrayMap` uses `arrayAppend` - ensure `arrayAppend` exists

### Test fails with "Can only call functions"

- BIF not registered in `register_all()`
- Check spelling (must be lowercase)
- Ensure function signature matches `BxNativeFunction` type

### Test fails with "Method not found"

- BIF not registered in `resolve_member_method()`
- Check method name mapping (case-insensitive)

### Behavior differs from BoxLang JVM

- Re-check JVM implementation for edge cases
- Look at JVM tests for expected behavior
- Check for type coercion differences

### Build fails

- Ensure all imports are correct
- Check function signature matches other BIFs
- Verify `BxValue` methods exist (new_string, new_number, etc.)

## Related Skills

- `reference-boxlang` - For locating and using BoxLang JVM reference
- `matchbox-compat` - For understanding MatchBox compatibility patterns
