# Watchdog tests

`tests/watchdog_contracts.rs` covers canonical evidence admission, foreign schema and inconsistent counts, invalid topology and dependency order preserving existing output, plus all four deterministic scenario outputs. It constructs a small governed DAG in memory without a downloaded corpus or backend.

Run `cargo nextest run --config-file configs/rust/nextest.toml --profile full --locked --offline -p bijux-dna-bench-hpc --all-targets`, and repeat with `--all-features`. CLI inline cases and `bench_local_dag_watchdog_simulation` preserve the original command behavior; `bench_local_hpc_submission_ready` exercises the readiness consumer. Strict lint and dependency boundary checks qualify the source independently.


The readiness contracts substitute an unavailable evidence provider without compiling the CLI. They retain all 99 diagnostic identities, prove continued independent assessment after provider failures, and preserve existing bytes when report-parent admission fails. A profile policy regression round-trips quoted and backslash-containing roots through the canonical TOML serializer. Original stage/pipeline mapping assertions remain with their declarations.

Dry-run owner contracts exercise provider failures, output refusal before acquisition, malformed persisted evidence and fresh acquisition during replay. The original public command test retains render, validation and drift rejection against local adapters.
