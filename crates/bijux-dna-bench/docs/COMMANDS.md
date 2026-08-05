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
| `load-stage-interface-catalog-from-repo` | `load_stage_interface_catalog_from_repo(path, repository_root)` | Load and validate one stage-interface catalog against an explicit repository root. |
| `load-checked-in-benchmark-contracts` | `load_checked_in_benchmark_contracts()` | Load the checked-in stage-campaign registry and benchmark portfolio while validating all checked-in planning inputs. |
| `load-checked-in-benchmark-planning-contracts` | `load_checked_in_benchmark_planning_contracts()` | Load the checked-in stage-campaign registry, benchmark portfolio, and stage-interface catalog and validate their defined identities. |
| `load-stage-report` | `load_stage_report(path)` | Load one stage-report JSON file into a source- and payload-checksummed admission envelope. |
| `load-stage-reports-from-dir` | `load_stage_reports_from_dir(path, registry, interfaces)` | Admit a declared report directory as checksummed envelopes against supplied campaign and interface contracts. |
| `load-checked-in-stage-reports` | `load_checked_in_stage_reports()` | Load all checked-in stage reports and validate campaign/interface alignment and unique stage ownership. |
| `derive-stage-coverage` | `derive_stage_coverage(registry, catalog, reports, required_commit)` | Derive typed retained-stage definition and admitted-evidence coverage with exact catalog tool closure and current-source freshness. |
| `publish-checked-in-stage-coverage` | `publish_checked_in_stage_coverage(repository_root, mode)` | Deterministically write or check the checked-in JSON, Markdown, and generated schema publications. |
| `plan-stage-campaign-attempts` | `plan_stage_campaign_attempts(registry, repository_commit, campaign_registry_sha256, run_id, campaign_id, attempt_id, observed_at_utc)` | Deterministically derive a source-locked plan with one validated lifecycle record per declared campaign tool. |
| `verify-campaign-plan-source` | `verify_campaign_plan_source(plan, repository_commit, campaign_registry_sha256)` | Reject a campaign plan unless its source commit and declaration digest match the executing checkout exactly. |
| `ensure-stage-campaign-attempts` | `ensure_stage_campaign_attempts(root, plan)` | Validate source locks, then preflight and crash-safely initialize every immutable planned attempt in a campaign cycle. |
| `benchmark-attempt-directory` | `benchmark_attempt_directory(root, identity)` | Derive the permanent `run-<id>-<domain>-<stage>/<tool>/attempts/<campaign>/<attempt>` storage path from validated identity. |
| `initialize-attempt-lifecycle` | `initialize_attempt_lifecycle(root, record)` | Delegate atomic publication of an immutable planned attempt record to `bijux-dna-runtime`. |
| `append-attempt-lifecycle-transition` | `append_attempt_lifecycle_transition(root, identity, transition)` | Delegate validation and no-clobber publication of one sequence-addressed transition to `bijux-dna-runtime`. |
| `load-attempt-lifecycle` | `load_attempt_lifecycle(root, identity)` | Delegate reconstruction and validation of immutable lifecycle evidence to `bijux-dna-runtime`. |
| `plan-attempt-migration` | `plan_attempt_migration(study_root, created_at_utc)` | Classify historical attempts by retained `run.kind` and produce a byte inventory bound plan. |
| `apply-attempt-migration` | `apply_attempt_migration(study_root, plan_path, ledger_path, updated_at_utc)` | Move planned attempts by same-filesystem rename while maintaining an interruption-safe ledger. |
| `verify-attempt-migration` | `verify_attempt_migration(study_root, plan_path, ledger_path)` | Recompute every current source or target file size and SHA-256 against the exact plan. |
| `rollback-attempt-migration` | `rollback_attempt_migration(study_root, plan_path, ledger_path, updated_at_utc)` | Verify and restore every applied attempt to its exact historical source path. |
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
| `bench-reports-dir` | `bench_reports_dir()` | Resolve the repository-owned benchmark report directory. |

## Suite Artifacts

Managed benchmark artifacts are:

- `observations.jsonl`
- `summary.json`
- `decision.json`
- `decisions.json`
- `migration-plan.json`
- `migration-ledger.json`
- `after-inventory.json`
- `run-<id>-<domain>-<stage>/<tool>/attempts/<campaign>/<attempt>/attempt.json`
- `run-<id>-<domain>-<stage>/<tool>/attempts/<campaign>/<attempt>/transitions/<sequence>.json`

`bijux-dna-bench` writes suite, summary, decision, and migration artifacts only
under declared benchmark output roots. Attempt lifecycle files are persisted and
loaded by `bijux-dna-runtime` through the compatibility exports above.

## Local Verification Commands

Run from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo check -p bijux-dna-bench --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench --test boundaries --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench --test contracts --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench --test determinism --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench --test semantics --no-default-features
```
