# Compiler Performance Benchmarks

This suite compares MatchBox and BoxLang process-level CPU and memory while
compiling and minimally executing representative source files.

The current CLIs do not expose a shared compile-only mode, so these benchmarks
use minimal execution and validate an expected output line for each fixture.
Treat the numbers as compiler-dominant process benchmarks, not pure parser or
bytecode timing.

## Fixtures

- `small_script.bxs`: small script with variables and expressions.
- `function_heavy.bxs`: many function declarations and calls.
- `class_heavy.bxs` plus `BenchPoint.bx`: class declaration, properties,
  methods, and `new`.
- `template_heavy.bxm`: template parsing with repeated output/interpolation.
- `generated/large_generated.bxs`: large generated script created by
  `generate-fixtures.sh`.

## Run

Build MatchBox release first so build time is not included:

```bash
cargo build --release
bash benchmarks/compiler/run.sh matchbox-release
```

Run BoxLang if it is on `PATH`:

```bash
bash benchmarks/compiler/run.sh boxlang
```

Run both engines:

```bash
bash benchmarks/compiler/run.sh both
```

Optional environment variables:

- `MATCHBOX_RELEASE_BIN`: path to the release MatchBox binary.
- `MATCHBOX_DEBUG_BIN`: path to the debug MatchBox binary.
- `BOXLANG_BIN`: BoxLang executable, default `boxlang`.
- `TIME_BIN`: time executable, default `/usr/bin/time`.

BoxLang may create or update files under `~/.boxlang`; make sure that directory
is writable before comparing cold runs.
