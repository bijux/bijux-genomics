# Benchmarks

This root holds tracked benchmark-owned assets and proof surfaces that must remain durable across
local validation and later benchmark completion work.

The committed subroots are:

- `benchmarks/configs/`
- `benchmarks/schemas/`
- `benchmarks/tests/`
- `benchmarks/readiness/`

Benchmark-owned config, schema, fixture, and deterministic readiness-proof consumers resolve from
this root through the shared benchmark path contract. The default root is `benchmarks/`,
`--benchmark-root` overrides it on the surfaces that expose that flag, and
`BIJUX_BENCHMARK_ROOT` provides the environment-level override when an explicit CLI root is not
passed.

Deterministic readiness proof now lives under the tracked benchmark root, while disposable run
products remain anchored under the active repository:

- `benchmarks/readiness/`
- `benchmarks/readiness/local-ready/`
- `runs/bench/local-smoke/`
- `target/local-fake-runs/`
- `target/slurm-dry-run/`
