# Watchdog tests

`tests/watchdog_contracts.rs` covers canonical evidence admission, foreign schema and inconsistent counts, invalid topology and dependency order preserving existing output, plus all four deterministic scenario outputs. It constructs a small governed DAG in memory without a downloaded corpus or backend.

Run `cargo nextest run --config-file configs/rust/nextest.toml --profile full --locked --offline -p bijux-dna-bench-hpc --all-targets`, and repeat with `--all-features`. CLI inline cases and `bench_local_dag_watchdog_simulation` preserve the original command behavior; `bench_local_hpc_submission_ready` exercises the readiness consumer. Strict lint and dependency boundary checks qualify the source independently.
