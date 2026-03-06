# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Portable runner stubs architecture for ultra-lean (~500KB) standalone native binaries.
- Cargo Workspace refactoring: separated `matchbox-vm`, `matchbox-compiler`, and `matchbox-runner`.
- Class inheritance support via `extends` attribute.
- Implicit accessors support (`accessors="true"`) for class properties.
- Optional type hints and access modifiers for functions and parameters.
- Default arguments support for function parameters.
- Interfaces with support for abstract methods, default implementations (traits), and multiple implementation.
- `onMissingMethod` magic method for dynamic method interception in classes.
- Semantic `.onError()` member method for asynchronous Futures.
- `arrayToList` BIF and member method mapping.
- Multi-target Native Fusion (Hybrid builds for Native, WASM, and JS).
- Dynamic JNI reflection bridge for Java interoperability.
- Persistent `BoxLangVM` for WASM with dynamic `call()` support.
- Automated JavaScript module generation via `--target js`.
- Member method delegation to BIFs (e.g., `"foo".ucase()`).
- High-performance integration testing macro (in-process execution).
- GitHub Actions for automated multi-platform Release and Snapshot builds.
- Tracing Mark-and-Sweep Garbage Collector.
- Hidden Classes (Shapes) and Monomorphic Inline Caches for performance.

### Changed
- Renamed project from `bx-rust` to `MatchBox`.
- Refactored binary into a library/binary hybrid.
- Updated GitHub workflows to support workspace building and runner stub injection.
- Optimized "Fusion" build generator to use `matchbox-vm` directly with release-profile optimizations.

### Fixed
- Fixed critical stack management bug in `OpInvoke` causing panics on certain method calls.
- Fixed parser panic on empty anonymous function parameters.
- Fixed case-insensitive function lookup in WASM/JS bridge.
- Fixed greedy parsing issue with optional identifiers preceding keywords in function declarations.
- Fixed various WASM runtime errors.
- Object lifetime issues in JNI bridge for release builds.
- Mac runner pool assignment errors in GitHub Actions.
