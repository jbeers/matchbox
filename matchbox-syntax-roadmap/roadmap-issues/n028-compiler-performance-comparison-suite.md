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

- [ ] Add compiler benchmark fixtures covering small scripts, large generated scripts, function-heavy code, class/component-heavy code, and template-heavy code
- [ ] Provide a repeatable command or script that runs MatchBox release against each fixture and captures wall time, user time, system time, and max RSS
- [ ] Provide a repeatable command or script that runs BoxLang against equivalent fixtures and captures the same metrics
- [ ] Separate compile-time measurements from runtime execution as much as the current CLIs allow; document any unavoidable runtime overhead
- [ ] Validate each fixture's output so failed compilation or execution cannot be mistaken for a fast benchmark
- [ ] Produce a concise comparison table for MatchBox release, MatchBox debug if useful, and BoxLang
- [ ] Document how to run the suite locally, including any required BoxLang filesystem permissions for `~/.boxlang`
- [ ] Ensure the benchmark suite can be extended with new fixtures without duplicating the runner logic

## Notes

- Prefer fixtures that stress compiler behavior rather than runtime-heavy loops.
- Include both cold process runs and repeated same-process runs if MatchBox gains
  a compile-only or reusable compiler entry point.
- If BoxLang does not expose a clean compile-only CLI path, record that and use a
  minimal-execution benchmark as the comparable fallback.
