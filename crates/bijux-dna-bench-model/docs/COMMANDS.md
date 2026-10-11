# Commands

This file is the SSOT for callable operations owned by
`bijux-dna-bench-model`.

## Managed Model Operations

These operations are pure Rust functions and type constructors; this crate does
not own CLI commands or process execution.

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `validate-all-domain-expected-result-rows` | `validate_all_domain_expected_result_rows` | Admit expected result projection against acquired source counts and governed baseline outputs, metrics and report sections. |
| `validate-all-domain-rendered-command-rows` | `validate_all_domain_rendered_command_rows` | Admit complete, unique governed result bindings with executable command steps and required adapter sources. |
| `validate-all-domain-stage-tool-table-rows` | `validate_all_domain_stage_tool_table_rows` | Admit unique stage/tool coverage and required columns against acquired domain-ready bindings and VCF row counts. |
| `admit-real-smoke-core-subset` | `admit_real_smoke_core_subset_report` | Admit required smoke stage/bridge coverage, metric presence and attached manifest outcomes. |
| `validate-local-pipeline-config` | `validate_benchmark_local_pipeline_dag_config` | Admit local pipeline configuration version and required identity/node envelope. |
| `local-pipeline-config-schemas` | `benchmark_local_pipeline_dag_config_schema`, `benchmark_local_pipeline_dag_metadata_config_schema`, `benchmark_local_pipeline_dag_stages_config_schema` | Generate strict combined and split configuration input schemas. |
| `validate-suite-with-catalog` | `validate_suite_with_catalog` | Validate suite structure and governed bindings using an explicit catalog provider. |
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
| `validate-container-report-portfolio-index` | `contract::validate_container_report_portfolio_index` | Validate report identity, checksummed file closure, summary counts, and conservative freshness promotion. |
| `container-report-portfolio-index-schema` | `container_report_portfolio_index_schema` | Generate the persisted container report-portfolio JSON Schema from its Rust model. |
| `validate-appraisal-report` | `contract::validate_appraisal_report` | Validate persisted appraisal identity, finding vocabulary and order, uniqueness, and exact summary closure. |
| `appraisal-report-schema` | `appraisal_report_schema` | Generate the persisted benchmark-appraisal JSON Schema from its Rust model. |
| `validate-hardening-queue-report` | `contract::validate_hardening_queue_report` | Validate deterministic queue allocation, canonical ordering, and grouped row/appraiser links. |
| `hardening-queue-report-schema` | `hardening_queue_report_schema` | Generate the persisted hardening-queue JSON Schema from its Rust model. |
| `validate-cross-benchmark-campaign-report` | `contract::validate_cross_benchmark_campaign_report` | Validate cross-domain campaign identity, selected-goal closure, nested evidence links, status semantics, and exact summary totals. |
| `cross-benchmark-campaign-report-schema` | `cross_benchmark_campaign_report_schema` | Generate the persisted cross-domain benchmark-campaign JSON Schema from its Rust model. |
| `validate-domain-benchmark-campaign-reports` | `validate_fastq_benchmark_campaign_report`, `validate_bam_benchmark_campaign_report`, `validate_vcf_benchmark_campaign_report`, `validate_hardening_benchmark_campaign_report` | Validate domain campaign identity, selected-goal closure, nested evidence links, status semantics, and exact summary totals. |
| `domain-benchmark-campaign-report-schemas` | `fastq_benchmark_campaign_report_schema`, `bam_benchmark_campaign_report_schema`, `vcf_benchmark_campaign_report_schema`, `hardening_benchmark_campaign_report_schema` | Generate the four persisted domain and hardening campaign schemas from their Rust models. |
| `validate-benchmark-matrix-report` | `contract::validate_benchmark_matrix_report` | Validate persisted matrix identity, unique rows, readiness semantics, surface matches, and exact summary closure. |
| `benchmark-matrix-report-schema` | `benchmark_matrix_report_schema` | Generate the persisted benchmark-matrix JSON Schema from its Rust model. |
| `validate-governed-data-catalog-snapshot-publication` | `contract::validate_governed_data_catalog_snapshot_publication` | Validate the versioned candidate-registry path, exact preserved/retained count closure, and embedded governed catalog registry. |
| `validate-governed-data-catalog-registry-publication` | `contract::validate_governed_data_catalog_registry_publication` | Validate the versioned immutable registry path, digest, and embedded governed catalog registry. |
| `validate-benchmark-campaign-report-publication` | `contract::validate_benchmark_campaign_report_publication` | Validate the versioned report paths, digests, campaign identity, presentation source, and overlay set. |
| `publication-outcome-schemas` | `governed_data_catalog_snapshot_publication_schema`, `governed_data_catalog_registry_publication_schema`, `benchmark_campaign_report_publication_schema` | Generate schemas for the persisted outcomes returned by repository and campaign publication workflows. |
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
| `validate-cycle-preparation` | `contract::validate_cycle_preparation` | Validate one versioned non-barrier preparation pass, including exact attempt, execution-spec, status, and outcome consistency. |
| `validate-cycle-admission` | `contract::validate_cycle_admission` | Validate one versioned non-barrier evidence-admission pass, including exact attempt, evidence, status, and outcome consistency. |
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

Suite catalog admission rejects missing input, output or parameter contracts when a declared stage uses that boundary. Empty declared identifier sets reject supplied values; unused contracts are not required. Executor and planner stages outside the supplied catalog retain their existing admission path.
