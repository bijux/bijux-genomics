# Commands

This file is the SSOT for commands and callable operations owned by
`bijux-dna-bench`.

## Managed Benchmark Operations

These operations are exported through the crate root and `public_api`.

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `load-suite` | `load_suite(path)` | Load and validate a benchmark suite TOML file. |
| `load-corpus-manifest` | `load_corpus_manifest(path)` | Load and validate one benchmark corpus TOML manifest. |
| `load-corpus-catalog` | `load_corpus_catalog()` | Load and validate all checked-in benchmark corpus manifests. |
| `load-bundle-manifest` | `load_bundle_manifest(path)` | Load and validate one benchmark bundle TOML manifest. |
| `load-bundle-catalog` | `load_bundle_catalog()` | Load and validate all checked-in benchmark bundle manifests. |
| `load-stage-campaign-registry` | `load_stage_campaign_registry(path)` | Load and validate one benchmark stage-campaign registry TOML file. |
| `load-benchmark-portfolio` | `load_benchmark_portfolio(path)` | Load and validate one benchmark portfolio TOML file. |
| `load-stage-interface-catalog` | `load_stage_interface_catalog(path)` | Load and validate one stage-interface TOML catalog and its repository-owned parameter and source contracts. |
| `load-checked-in-benchmark-contracts` | `load_checked_in_benchmark_contracts()` | Load the checked-in stage-campaign registry and benchmark portfolio while validating all checked-in planning inputs. |
| `load-checked-in-benchmark-planning-contracts` | `load_checked_in_benchmark_planning_contracts()` | Load the checked-in stage-campaign registry, benchmark portfolio, and stage-interface catalog and validate their defined identities. |
| `summarize` | `summarize(suite, observations, options)` | Build deterministic benchmark summaries from observations. |
| `compare` | `compare(summary_a, summary_b)` | Compare two completed benchmark summaries. |
| `gate` | `gate(policy, summary)` | Evaluate benchmark rows against a gate policy. |
| `bench-data-dir` | `bench_data_dir()` | Resolve the crate-owned benchmark data directory. |
| `bench-suites-dir` | `bench_suites_dir()` | Resolve the crate-owned checked-in suite catalog directory. |
| `bench-corpora-dir` | `bench_corpora_dir()` | Resolve the crate-owned checked-in corpus catalog directory. |
| `bench-bundles-dir` | `bench_bundles_dir()` | Resolve the crate-owned checked-in bundle catalog directory. |
| `bench-contracts-dir` | `bench_contracts_dir()` | Resolve the repository-owned benchmark contract directory. |
| `bench-stage-campaigns-path` | `bench_stage_campaigns_path()` | Resolve the repository-owned stage-campaign registry path. |
| `bench-portfolio-path` | `bench_portfolio_path()` | Resolve the repository-owned benchmark portfolio path. |
| `bench-stage-interfaces-path` | `bench_stage_interfaces_path()` | Resolve the repository-owned stage-interface catalog path. |

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
