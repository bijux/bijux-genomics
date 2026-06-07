# Benchmarks

This root holds tracked benchmark-owned assets and proof surfaces that must remain durable across
local validation and later benchmark completion work.

The committed subroots are:

- `benchmarks/configs/`
- `benchmarks/schemas/`
- `benchmarks/tests/`
- `benchmarks/readiness/`

Benchmark-owned config, schema, and fixture consumers resolve from this root through the shared
benchmark path contract. The default root is `benchmarks/`, `--benchmark-root` overrides it on the
surfaces that expose that flag, and `BIJUX_BENCHMARK_ROOT` provides the environment-level override
when an explicit CLI root is not passed.

Readiness reports and disposable run products are still anchored under the active repository:

- `target/bench-readiness/`
- `target/local-ready/`
- `target/local-fake-runs/`
- `target/slurm-dry-run/`
