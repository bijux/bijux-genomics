# Commands

This file is the SSOT for commands and callable operations owned by
`bijux-dna-bench`.

## Managed Benchmark Operations

These operations are exported through the crate root and `public_api`.

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `load-suite` | `load_suite(path)` | Load and validate a benchmark suite TOML file. |
| `summarize` | `summarize(suite, observations, options)` | Build deterministic benchmark summaries from observations. |
| `compare` | `compare(summary_a, summary_b)` | Compare two completed benchmark summaries. |
| `gate` | `gate(policy, summary)` | Evaluate benchmark rows against a gate policy. |
| `bench-data-dir` | `bench_data_dir()` | Resolve the crate-owned benchmark data directory. |
| `bench-suites-dir` | `bench_suites_dir()` | Resolve the crate-owned checked-in suite catalog directory. |

## Suite Artifacts

Managed benchmark artifacts are:

- `observations.jsonl`
- `summary.json`
- `decision.json`
- `decisions.json`

`bijux-dna-bench` writes and reads benchmark artifacts only under declared
benchmark output roots.

## Local Verification Commands

Run from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo check -p bijux-dna-bench --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench --test boundaries --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench --test contracts --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench --test determinism --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench --test semantics --no-default-features
```
