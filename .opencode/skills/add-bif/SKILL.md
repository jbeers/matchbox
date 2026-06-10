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
3. BoxLang JVM repository available

**IMPORTANT: Load the `reference-boxlang` skill before starting Step 1.** It provides essential guidance on locating and reading the BoxLang JVM source code, which defines the canonical behavior for all BIFs.

## MatchBox Value System

Understanding MatchBox's value representation is critical before implementing any BIF.

### NaN-Boxed Values

`BxValue` is a single `u64` using NaN-boxing (NOT an enum). All types fit in 64 bits:

```rust
pub struct BxValue(u64);  // NOT an enum!
```

Key methods on `BxValue`:
- `BxValue::new_ptr(gc_id)` - Create a pointer to a GC heap object
- `BxValue::new_bool(b)` - Create a boolean
- `BxValue::new_null()` - Create null
- `val.as_gc_id() -> Option<usize>` - Extract GC heap index (for structs, arrays, strings)
- `val.is_ptr()` - Check if this is a heap pointer
- `val.is_null()` - Check if null
- `val.as_number() -> f64` - Extract numeric value

### GC Heap Objects

Pointers reference `GcObject` variants on the GC heap:

```rust
pub enum GcObject {
    String(BoxString),
    Array(Vec<BxValue>),
    Struct(BxStruct),
    DateTime(DateTime<Utc>),
    // ... etc
}
```

To determine the type of a pointed-to value, use the `BxVM` trait methods:
- `vm.is_struct_value(val)` - Check if a BxValue points to a struct
- `vm.is_array_value(val)` - Check if a BxValue points to an array
- `vm.is_string_value(val)` - Check if a BxValue points to a string

### BxVM Trait Methods

The `BxVM` trait provides all VM operations. Key methods by type:

**Strings:** `string_new(String) -> usize`, `to_string(BxValue) -> String`

**Arrays:** `array_new() -> usize`, `array_len(id) -> usize`, `array_get(id, idx) -> BxValue`, `array_push(id, val)`, `array_set(id, idx, val)`

**Structs:** `struct_new() -> usize`, `struct_len(id) -> usize`, `struct_get(id, key) -> BxValue`, `struct_set(id, key, val)`, `struct_delete(id, key) -> bool`, `struct_key_exists(id, key) -> bool`, `struct_key_array(id) -> Vec<String>`, `struct_clear(id)`

**Type checks:** `is_struct_value(val) -> bool`, `is_array_value(val) -> bool`

### MatchBox Design Decisions

These affect BIF behavior and differ from BoxLang JVM:

- **Structs are always case-insensitive** - The string interner lowercases all keys. `structIsCaseSensitive()` always returns `false`.
- **Structs always maintain insertion order** - `structIsOrdered()` always returns `true`.
- **Missing struct keys return null** - `struct_get()` returns `BxValue::new_null()` for missing keys (no error).
- **BIF names are registered in lowercase** - `bifs.insert("mybif".to_string(), ...)` - BoxLang is case-insensitive.
- **Method names in `resolve_member_method` are matched lowercase** - The VM lowercases the method name before matching.

## Workflow

### Step 1: Research the BIF in BoxLang JVM

**Load the `reference-boxlang` skill first** for guidance on locating BoxLang source code.

Then read BOTH the implementation AND the tests:

```bash
# Find and read the BIF implementation
find ~/dev/ortus-boxlang/BoxLang/src/main/java/ortus/boxlang/runtime/bifs -name "*<BifName>*.java"
cat <path-to-bif>.java

# Find and read the tests - these reveal expected behavior and edge cases
find ~/dev/ortus-boxlang/BoxLang/src/test/java -name "*<BifName>*Test.java"
cat <path-to-test>.java
```

Key things to note:
- What arguments does it take? (names, types, required vs optional, defaults)
- What does it return?
- How does it handle edge cases (null, empty, invalid input)?
- Is it also a member method on structs/arrays/strings? (check for `@BoxMember` annotation)
- Does it recurse into nested structures?
- What utility methods does it call? (e.g., `StructUtil.findKey()`)

### Step 2: Write Test (RED Phase)

Create a test script in `tests/scripts/`:

```boxlang
// Test <bif_name> function
var result1 = <bif_name>( "arg1", "arg2" );
if ( result1 != expected1 ) { throw "<bif_name> basic test failed: got " & result1; }

var result2 = <bif_name>( "edge_case" );
if ( result2 != expected2 ) { throw "<bif_name> edge case failed: got " & result2; }

// Test as member method if applicable (check @BoxMember in JVM source)
var s = { name: "test" };
if ( !s.<methodName>( "arg" ) ) { throw "<bif_name> method failed"; }

println( "<bif_name> OK" );
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
    
    Ok(BxValue::new_ptr(vm.string_new(result)))
}
```

Register the BIF in the `register_all()` function:

```rust
bifs.insert("<bif_name>".to_string(), <bif_name>_bif as BxNativeFunction);
```

#### Register as Member Method (if applicable)

If the BIF is a member method on a type (check `@BoxMember` annotation in JVM source), register it in `crates/matchbox-vm/src/vm/mod.rs` inside `resolve_member_method()`. Match against the **lowercased** method name:

```rust
// For struct methods - under GcObject::Struct(_) match arm:
"<method_name>" => Some("<bif_name>".to_string()),

// For array methods - under GcObject::Array(_) match arm:
"<method_name>" => Some("<bif_name>".to_string()),

// For string methods - under GcObject::String(_) match arm:
"<method_name>" => Some("<bif_name>".to_string()),
```

The method name in the match arm must be lowercase (the VM lowercases all method names before lookup). The BIF name it maps to must match exactly what was registered in `register_all()`.

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

### Step 5: Update BIF Status

Update `BIF_STATUS.md` to mark the BIF as implemented. Update both the individual BIF row (change ⬜ to ✅) and the section summary counts.

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
    
    Ok(BxValue::new_ptr(vm.string_new(result)))
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

### Struct BIFs

```rust
fn struct_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("structBif() expects 1 argument".to_string());
    }
    
    let id = args[0].as_gc_id()
        .ok_or("structBif() expects a struct as the first argument")?;
    
    // Read keys
    let keys = vm.struct_key_array(id);
    
    // Read a value
    let val = vm.struct_get(id, "someKey");
    
    // Build a new struct as result
    let result_id = vm.struct_new();
    for key in &keys {
        let v = vm.struct_get(id, key);
        vm.struct_set(result_id, key, v);
    }
    
    Ok(BxValue::new_ptr(result_id))
}
```

### Struct BIF Returning Metadata (new struct with specific keys)

```rust
fn struct_metadata_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("structMetadata() expects 1 argument".to_string());
    }
    if args[0].as_gc_id().is_none() {
        return Err("structMetadata() expects a struct".to_string());
    }
    let meta_id = vm.struct_new();
    vm.struct_set(meta_id, "casesensitive", BxValue::new_bool(false));
    vm.struct_set(meta_id, "ordered", BxValue::new_bool(true));
    let type_str = vm.string_new("linked".to_string());
    vm.struct_set(meta_id, "type", BxValue::new_ptr(type_str));
    Ok(BxValue::new_ptr(meta_id))
}
```

### Recursive Struct Search (e.g., structFindKey)

```rust
fn struct_find_key_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    let id = args[0].as_gc_id()
        .ok_or("structFindKey() expects a struct")?;
    let search_key = vm.to_string(args[1]);
    let scope_all = args.len() >= 3 && vm.to_string(args[2]).eq_ignore_ascii_case("all");
    
    let results_id = vm.array_new();
    let keys = vm.struct_key_array(id);
    
    for key in &keys {
        // Check if this key matches
        if key.eq_ignore_ascii_case(&search_key) {
            let val = vm.struct_get(id, key);
            let entry_id = vm.struct_new();
            vm.struct_set(entry_id, "owner", BxValue::new_ptr(id));
            vm.struct_set(entry_id, "value", val);
            vm.array_push(results_id, BxValue::new_ptr(entry_id));
            if !scope_all { break; }
        }
        // Recurse into nested structs
        let val = vm.struct_get(id, key);
        if let Some(nested_id) = val.as_gc_id() {
            if vm.is_struct_value(val) {
                // Recursive call...
            }
        }
    }
    Ok(BxValue::new_ptr(results_id))
}
```

## Common Edge Cases

### Null/Empty Handling

```rust
// BoxLang typically returns sensible defaults
if input.is_empty() {
    let empty = vm.string_new("".to_string());
    return Ok(BxValue::new_ptr(empty));
}
```

### Type Checking for GC Objects

```rust
// Always check as_gc_id() and verify the type
let id = args[0].as_gc_id()
    .ok_or("expects a struct as the first argument")?;

// For methods that accept multiple types, check explicitly:
if vm.is_struct_value(args[0]) {
    // handle struct
} else if vm.is_array_value(args[0]) {
    // handle array
}
```

### Case Insensitivity

```rust
// BIF names are registered in lowercase
bifs.insert("mybif".to_string(), my_bif as BxNativeFunction);

// Method names in resolve_member_method are matched lowercase
"mybif" => Some("mybif".to_string()),

// For case-insensitive string comparison in BIF logic:
key.eq_ignore_ascii_case(&search_key)
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

// Empty struct
var s = {};
var result = myBif( s );
if ( result != expected ) { throw "empty struct failed"; }
```

### Member Method Syntax

```boxlang
// Test both function and method forms
var result1 = myBif( s, "arg" );
var result2 = s.myMethod( "arg" );
if ( result1 != result2 ) { throw "function and method should match"; }
```

### Multiple Arguments

```boxlang
var result = myBif( "arg1", "arg2", "arg3" );
if ( result != "expected" ) { throw "multi-arg failed"; }
```

### Nested Structures

```boxlang
// Test with nested structs/arrays if BIF recurses
var nested = { a: { b: { c: 1 } } };
var result = myBif( nested, "c" );
if ( result != 1 ) { throw "nested failed"; }
```

## Checklist

Before submitting:

- [ ] Loaded `reference-boxlang` skill and researched BoxLang JVM implementation
- [ ] Read BOTH the JVM implementation AND the JVM tests
- [ ] Decided: prelude (BoxLang) vs native (Rust) implementation
- [ ] Wrote comprehensive tests (basic, edge cases, member method syntax, nested structures)
- [ ] Test fails before implementation (RED)
- [ ] Implementation matches BoxLang behavior
- [ ] Registered BIF in `register_all()` (lowercase name)
- [ ] Registered member method in `resolve_member_method()` if applicable (lowercase match arm)
- [ ] Test passes after implementation (GREEN)
- [ ] Full test suite passes (no regressions)
- [ ] Updated `BIF_STATUS.md` (individual row + section summary counts)

### Prelude-specific checklist:

- [ ] Can be implemented using existing primitives (no VM internals needed)
- [ ] Includes JSDoc-style documentation comment
- [ ] Uses camelCase naming convention
- [ ] Tested with tree-shaking enabled (default build)

## Examples from Recent Work

### structEquals (Native Rust BIF with recursive comparison)

```bash
# 1. Research in BoxLang (using reference-boxlang skill)
cat ~/dev/ortus-boxlang/BoxLang/src/main/java/ortus/boxlang/runtime/bifs/global/struct/StructEquals.java
cat ~/dev/ortus-boxlang/BoxLang/src/test/java/ortus/boxlang/runtime/bifs/global/struct/StructEqualsTest.java

# 2. Write test
cat > tests/scripts/vm_struct_equals.bxs << 'EOF'
var s1 = { name: "John", age: 30 };
var s2 = { name: "John", age: 30 };
var s3 = { name: "Jane", age: 30 };
if ( structEquals( s1, s2 ) != true ) { throw "equal structs failed"; }
if ( structEquals( s1, s3 ) != false ) { throw "unequal structs failed"; }
// Nested structs
var n1 = { person: { name: "John" } };
var n2 = { person: { name: "John" } };
if ( structEquals( n1, n2 ) != true ) { throw "nested equal failed"; }
// Method syntax
if ( s1.equals( s2 ) != true ) { throw "method failed"; }
println( "structEquals OK" );
EOF

# 3. Implement
# In crates/matchbox-vm/src/bifs/mod.rs:
fn struct_equals_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    let id1 = args[0].as_gc_id().ok_or("...")?;
    let id2 = args[1].as_gc_id().ok_or("...")?;
    // Compare lengths, keys, then values (recursing for nested structs/arrays)
    Ok(BxValue::new_bool(/* deep comparison */))
}

# Register:
bifs.insert("structequals".to_string(), struct_equals_bif as BxNativeFunction);

# In crates/matchbox-vm/src/vm/mod.rs, under GcObject::Struct(_):
"equals" => Some("structequals".to_string()),
```

### structIsCaseSensitive (Simple BIF reflecting MatchBox design)

```rust
// MatchBox structs are always case-insensitive, so this always returns false
fn struct_is_case_sensitive_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("structIsCaseSensitive() expects 1 argument".to_string());
    }
    if args[0].as_gc_id().is_some() {
        Ok(BxValue::new_bool(false))
    } else {
        Err("structIsCaseSensitive() expects a struct".to_string())
    }
}
```

### structToQueryString (BIF with URL encoding)

```rust
fn struct_to_query_string_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    let id = args[0].as_gc_id().ok_or("...")?;
    let delimiter = if args.len() >= 2 { vm.to_string(args[1]) } else { "&".to_string() };
    let keys = vm.struct_key_array(id);
    let mut parts = Vec::new();
    for key in &keys {
        let val = vm.struct_get(id, key);
        let val_str = vm.to_string(val);
        parts.push(format!("{}={}", percent_encode(key), percent_encode(&val_str)));
    }
    let qs = parts.join(&delimiter);
    Ok(BxValue::new_ptr(vm.string_new(qs)))
}
```

### arrayMap (Prelude BIF)

```boxlang
// Implement in crates/matchbox-compiler/src/prelude.bxs:
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
- Check method name is lowercase in the match arm
- Ensure it's under the correct `GcObject` variant (Struct, Array, String, etc.)
- The mapped BIF name must exactly match what's in `register_all()`

### Behavior differs from BoxLang JVM

- Re-check JVM implementation for edge cases
- Look at JVM tests for expected behavior
- Check for type coercion differences
- Remember MatchBox-specific decisions (structs always case-insensitive, always ordered)

### Build fails

- Ensure all imports are correct
- Check function signature matches other BIFs: `fn(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String>`
- Remember `BxValue` is NaN-boxed, not an enum - use `.as_gc_id()`, `.as_number()`, `.is_null()`, etc.
- Use `vm.string_new(String)` which returns `usize`, then wrap with `BxValue::new_ptr()`

## Related Skills

- `reference-boxlang` - **Load this first.** For locating and using BoxLang JVM reference
- `matchbox-compat` - For understanding MatchBox compatibility patterns
