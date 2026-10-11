# Commands

This file is the SSOT for callable operations owned by
`bijux-dna-bench-model`.

## Managed Model Operations

These operations are pure Rust functions and type constructors; this crate does
not own CLI commands or process execution.

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `validate-suite` | `contract::validate_suite` | Validate `BenchmarkSuiteSpec` shape, graph, stage ids, tool ids, and parameter bindings. |
| `validate-stage-campaign-registry` | `contract::validate_stage_campaign_registry` | Validate `BenchmarkStageCampaignRegistry` shape and campaign-level invariants. |
| `validate-stage-coverage` | `contract::validate_stage_coverage` | Validate version, ordering, identities, state consistency, and exact summary closure for a stage-coverage publication. |
| `benchmark-stage-coverage-schema` | `benchmark_stage_coverage_schema` | Generate the JSON Schema for the versioned stage-coverage publication from its Rust type. |
| `validate-stage-universe` | `contract::validate_stage_universe` | Validate stage ordering, explicit disposition rationale, permanent run reservations, and exact summary closure. |
| `benchmark-stage-universe-schema` | `benchmark_stage_universe_schema` | Generate the JSON Schema for the canonical stage-universe publication. |
| `validate-benchmark-portfolio` | `contract::validate_benchmark_portfolio` | Validate `BenchmarkPortfolio` declarations, corpora, profiles, and stage assignments. |
| `validate-defined-campaign-portfolio-alignment` | `contract::validate_defined_campaign_portfolio_alignment` | Validate that every defined stage campaign matches its reserved portfolio run, profile, and corpus identities. |
| `validate-portfolio-campaign-alignment` | `contract::validate_portfolio_campaign_alignment` | Validate full 1:1 closure between all reserved portfolio stages and the stage campaign registry. |
| `validate-stage-interface-catalog` | `contract::validate_stage_interface_catalog` | Validate stage interfaces, adapters, parameter ownership, downstream probes, and repository-relative contract paths. |
| `validate-pipeline-parameter-config` | `contract::validate_pipeline_parameter_config` | Validate the governed per-stage pipeline parameter configuration. |
| `validate-defined-interface-campaign-alignment` | `contract::validate_defined_interface_campaign_alignment` | Validate that every defined stage interface matches a campaign and its exact tool set. |
| `validate-interface-parameter-alignment` | `contract::validate_interface_parameter_alignment` | Validate parameter schema, defaults provider, and allowed overrides for one stage interface. |
| `validate-stage-report` | `contract::validate_stage_report` | Validate benchmark stage-report identity, execution design, tool evidence, comparability, metrics, and verdict. |
| `validate-stage-report-alignment` | `contract::validate_stage_report_alignment` | Validate a stage report against its campaign and optional stage-interface authority. |
| `validate-report-admission` | `contract::validate_report_admission` | Validate exact source bytes, canonical parsed-report identity, and the nested stage-report contract. |
| `validate-benchmark-run-definition` | `contract::validate_benchmark_run_definition` | Validate one versioned permanent run protocol and its explicit supersession lineage. |
| `validate-benchmark-run-definition-revision` | `contract::validate_benchmark_run_definition_revision` | Require a same-run protocol version increase or a replacement run that explicitly supersedes the prior semantic stage protocol. |
| `benchmark-run-definition-sha256` | `contract::benchmark_run_definition_sha256` | Derive the deterministic canonical SHA-256 identity of a valid run protocol. |
| `benchmark-run-directory-name` | `contract::benchmark_run_directory_name` | Derive `run-<id>-<domain>-<stage>` only from a validated permanent run identity. |
| `validate-attempt-identity` | `contract::validate_attempt_identity` | Validate permanent campaign, run, attempt, and tool identity before path derivation or persistence. |
| `benchmark-attempt-identity-schema` | `benchmark_attempt_identity_schema` | Generate the standalone scheduler-independent attempt identity schema. |
| `benchmark-attempt-identity-sha256` | `contract::benchmark_attempt_identity_sha256` | Derive the deterministic canonical SHA-256 identity of a valid attempt. |
| `validate-attempt-record-v2` | `contract::validate_attempt_record_v2` | Validate permanent attempt identity, append-only transition order, Slurm jobs, terminal accounting, checksummed evidence, and terminal admission. |
| `append-attempt-transition` | `contract::append_attempt_transition` | Append a legal transition without mutating the record when validation fails. |
| `validate-cycle-status-v2` | `contract::validate_cycle_status_v2` | Validate cycle identity, retry lineage, and single-active-attempt ownership per run and tool. |
| `validate-attempt-archive` | `contract::validate_attempt_archive` | Validate identity-complete retained evidence for a failed or superseded historical attempt. |
| `validate-observation` | `contract::validate_observation` | Validate one `BenchmarkObservation` before summarization. |
| `validate-summary` | `contract::validate_summary` | Validate a `BenchmarkSummary` before persistence or comparison. |
| `validate-decision` | `contract::validate_decision` | Validate a `GateDecision` before persistence. |
| `compare-summaries` | `compare::compare_summaries` | Build a deterministic typed comparison report from two summaries. |
| `robust-stats` | `stats::robust_stats` | Compute deterministic robust statistics. |
| `bootstrap-ci` | `stats::bootstrap_ci` | Compute seeded bootstrap confidence intervals. |
| `mad-outliers` | `stats::mad_outliers` | Detect deterministic median-absolute-deviation outliers. |

Scientific gate evaluation is owned by
`bijux_dna_science::decision::evaluate_benchmark_gate_policy`; this crate owns
the typed policy inputs and decision records consumed by that evaluator.

## Local Verification Commands

Run from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo check -p bijux-dna-bench-model --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test boundaries --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test contracts --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test determinism --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test schemas --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test semantics --no-default-features
```
