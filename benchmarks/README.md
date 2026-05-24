# QoQ 1M Row Benchmark

This benchmark builds a 1,000,000 row in-memory query and then runs `AVG(value)` over it.
It is intended for comparing MatchBox and BoxLang on the same workload.

## Run MatchBox

```bash
/usr/bin/time -v cargo run --features qoq -- benchmarks/qoq_avg_1m.bxs
```

If you already have a local `matchbox` binary installed:

```bash
/usr/bin/time -v matchbox benchmarks/qoq_avg_1m.bxs
```

## Run BoxLang

```bash
/usr/bin/time -v boxlang benchmarks/qoq_avg_1m.bxs
```

## Notes

- `build ms` is the time to create the 1,000,000 row query.
- `qoq ms` is the time for the `AVG(value)` QoQ execution.
- `/usr/bin/time -v` gives peak RSS for the full process, which is the simplest memory comparison across runtimes.
- Run each command a few times and compare the steady-state numbers, not the first cold run only.
