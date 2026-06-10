# MatchBox Agent Instructions

## What is MatchBox

MatchBox is a custom VM for BoxLang, a spiritual successor to CFML. It aims for compatibility with the BoxLang JVM reference implementation while targeting environments where the JVM struggles: WebAssembly, ESP32, cloud functions, and native binaries.

The VM includes tiered JIT compilation (Cranelift backend) for hot code paths, NaN-boxed values for efficient memory use, and generational garbage collection.

## Commands

```bash
cargo build                    # Build
cargo test                     # Run all tests
cargo test vm_array_bifs       # Run specific test
cargo clippy                   # Lint
cargo build --release --features "bif-http,bif-zip"  # Verify release build
```

## Project Structure

- `crates/matchbox-vm/src/bifs/mod.rs` — Native Rust BIFs
- `crates/matchbox-compiler/src/prelude.bxs` — Prelude BIFs (BoxLang)
- `tests/scripts/` — Integration test scripts (.bxs)
- `tests/integration_tests.rs` — Test registration
- `reference/boxlang/` — BoxLang JVM reference (read-only)
- `BIF_STATUS.md` — BIF implementation tracking

## Adding BIFs

Load the `add-bif` skill. It covers:
- Prelude vs native decision tree
- Implementation patterns
- Testing workflow
- Registration steps

## BoxLang Reference

Load the `reference-boxlang` skill. BoxLang JVM defines canonical behavior. Always check it first.
