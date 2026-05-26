# N028: Compiler Performance Comparison Suite

**Type:** Tooling / Performance
**Priority:** Medium

## What to build

Create a repeatable benchmark suite for comparing MatchBox compiler CPU time and
memory usage against the BoxLang compiler on representative source files.

The suite should mirror the QoQ benchmark workflow: build or select benchmark
fixtures, run both runtimes with `/usr/bin/time -v`, validate that the compiled
programs behave correctly, and report wall time plus maximum RSS in a format that
is easy to paste into roadmap or PR notes.

## Why

QoQ benchmarking exposed real runtime bottlenecks and made regressions visible.
The compiler needs the same kind of comparison harness so parser, CST, AST, and
bytecode work can be evaluated against BoxLang with CPU and memory data instead
of intuition.

## Acceptance criteria

- [x] Add compiler benchmark fixtures covering small scripts, large generated scripts, function-heavy code, class/component-heavy code, and template-heavy code
- [x] Provide a repeatable command or script that runs MatchBox release against each fixture and captures wall time, user time, system time, and max RSS
- [x] Provide a repeatable command or script that runs BoxLang against equivalent fixtures and captures the same metrics
- [x] Separate compile-time measurements from runtime execution as much as the current CLIs allow; document any unavoidable runtime overhead
- [x] Validate each fixture's output so failed compilation or execution cannot be mistaken for a fast benchmark
- [x] Produce a concise comparison table for MatchBox release, MatchBox debug if useful, and BoxLang
- [x] Document how to run the suite locally, including any required BoxLang filesystem permissions for `~/.boxlang`
- [x] Ensure the benchmark suite can be extended with new fixtures without duplicating the runner logic

## Status

Implemented in `benchmarks/compiler`.

The suite includes reusable fixtures, a deterministic large-fixture generator,
and `run.sh`, which emits a markdown table with wall time, user time, system
time, max RSS, and output-validation status for each engine/fixture pair.

The current MatchBox and BoxLang CLIs do not expose a shared compile-only mode,
so this suite measures process-level compile plus minimal execution. Fixture
runtime work is intentionally tiny so parser/compiler overhead dominates.

Initial local release comparison:

| engine | fixture | wall | user | sys | max_rss_kb | status |
|---|---|---:|---:|---:|---:|---|
| matchbox-release | small | 0:00.01 | 0.00 | 0.00 | 20200 | ok |
| matchbox-release | large-generated | 0:00.01 | 0.00 | 0.00 | 20108 | ok |
| matchbox-release | function-heavy | 0:00.01 | 0.00 | 0.00 | 20492 | ok |
| matchbox-release | class-heavy | 0:00.01 | 0.00 | 0.01 | 20300 | ok |
| matchbox-release | template-heavy | 0:00.01 | 0.00 | 0.01 | 20292 | ok |
| boxlang | small | 0:00.38 | 0.65 | 0.11 | 98812 | ok |
| boxlang | large-generated | 0:00.52 | 1.03 | 0.15 | 115676 | ok |
| boxlang | function-heavy | 0:00.39 | 0.67 | 0.10 | 98584 | ok |
| boxlang | class-heavy | 0:00.48 | 0.85 | 0.09 | 108732 | ok |
| boxlang | template-heavy | 0:00.46 | 0.79 | 0.13 | 104380 | ok |

## Notes

- Prefer fixtures that stress compiler behavior rather than runtime-heavy loops.
- Include both cold process runs and repeated same-process runs if MatchBox gains
  a compile-only or reusable compiler entry point.
- If BoxLang does not expose a clean compile-only CLI path, record that and use a
  minimal-execution benchmark as the comparable fallback.
