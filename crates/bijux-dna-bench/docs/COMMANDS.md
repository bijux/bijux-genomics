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
| `load-stage-interface-catalog-for-publication` | `load_stage_interface_catalog_for_publication(path, source_repository_root, presentation_repository_root)` | Complete only absent historical report-presentation tables from an explicit validated presentation authority. |
| `load-checked-in-benchmark-contracts` | `load_checked_in_benchmark_contracts()` | Load the checked-in stage-campaign registry and benchmark portfolio while validating all checked-in planning inputs. |
| `load-checked-in-benchmark-planning-contracts` | `load_checked_in_benchmark_planning_contracts()` | Load the checked-in stage-campaign registry, benchmark portfolio, and stage-interface catalog and validate their defined identities. |
| `derive-benchmark-matrix` | `derive_benchmark_matrix(request)` | Derive the selected stage/tool frontier from all supported catalog pairs, retaining external deliveries and explicit refusal rows for contract drift. |
| `load-benchmark-matrix-report` | `load_benchmark_matrix_report(path)` | Parse and admit a persisted benchmark matrix before product workflow use. |
| `publish-benchmark-matrix` | `publish_benchmark_matrix(path, report)` | Validate the complete matrix identity, surface, readiness, and summary closure before atomic publication. |
| `current-benchmark-matrix-timestamp` | `current_benchmark_matrix_timestamp()` | Supply an explicit generation timestamp for thin operator requests. |
| `appraise-matrix-report` | `appraise_matrix_report(matrix)` | Run every registered product appraiser and validate canonical finding and summary closure. |
| `derive-hardening-queue` | `hardening_queue_from_appraisal(report)` | Group admitted findings into a deterministic contiguous hardening queue. |
| `load-appraisal-products` | `load_appraisal_report(path)`, `load_hardening_queue_report(path)` | Parse and validate persisted appraisal and hardening-queue records. |
| `publish-appraisal-products` | `publish_appraisal_report(path, report)`, `publish_hardening_queue_report(path, report)` | Validate and atomically publish appraisal and hardening-queue records. |
| `derive-cross-benchmark-campaign` | `derive_cross_benchmark_campaign(matrix, appraisal, queue, requested_goals)` | Derive a cross-domain goal campaign from admitted matrix and appraisal products. |
| `load-cross-benchmark-campaign` | `load_cross_benchmark_campaign_report(path)` | Parse and admit a persisted cross-domain campaign report. |
| `publish-cross-benchmark-campaign` | `publish_cross_benchmark_campaign_report(path, report)` | Validate and atomically publish a cross-domain campaign report. |
| `derive-bam-benchmark-campaign` | `derive_bam_benchmark_campaign(matrix, appraisal, queue, requested_goals)` | Derive a BAM goal campaign from admitted matrix and appraisal products. |
| `load-bam-benchmark-campaign` | `load_bam_benchmark_campaign_report(path)` | Parse and admit a persisted BAM campaign report. |
| `publish-bam-benchmark-campaign` | `publish_bam_benchmark_campaign_report(path, report)` | Validate and atomically publish a BAM campaign report. |
| `derive-fastq-benchmark-campaign` | `derive_fastq_benchmark_campaign(matrix, appraisal, queue, requested_goals)` | Derive a FASTQ goal campaign from admitted matrix and appraisal products. |
| `load-fastq-benchmark-campaign` | `load_fastq_benchmark_campaign_report(path)` | Parse and admit a persisted FASTQ campaign report. |
| `publish-fastq-benchmark-campaign` | `publish_fastq_benchmark_campaign_report(path, report)` | Validate and atomically publish a FASTQ campaign report. |
| `derive-vcf-benchmark-campaign` | `derive_vcf_benchmark_campaign(matrix, appraisal, queue, requested_goals)` | Derive a VCF goal campaign from admitted matrix and appraisal products. |
| `load-vcf-benchmark-campaign` | `load_vcf_benchmark_campaign_report(path)` | Parse and admit a persisted VCF campaign report. |
| `publish-vcf-benchmark-campaign` | `publish_vcf_benchmark_campaign_report(path, report)` | Validate and atomically publish a VCF campaign report. |
| `derive-hardening-benchmark-campaign` | `derive_hardening_benchmark_campaign(matrix, appraisal, queue, requested_goals)` | Derive a portfolio hardening goal campaign from admitted matrix and appraisal products. |
| `load-hardening-benchmark-campaign` | `load_hardening_benchmark_campaign_report(path)` | Parse and admit a persisted hardening campaign report. |
| `publish-hardening-benchmark-campaign` | `publish_hardening_benchmark_campaign_report(path, report)` | Validate and atomically publish a hardening campaign report. |
| `load-stage-report` | `load_stage_report(path)` | Load one stage-report JSON file into a source- and payload-checksummed admission envelope. |
| `load-stage-reports-from-dir` | `load_stage_reports_from_dir(path, registry, interfaces)` | Admit a declared report directory as checksummed envelopes against supplied campaign and interface contracts. |
| `load-benchmark-stage-result-manifest` | `load_benchmark_stage_result_manifest(path)` | Parse and structurally validate one persisted stage execution result under benchmark ownership. |
| `validate-benchmark-stage-result-path` | `validate_benchmark_stage_result_path(repository_root, path)` | Derive the stable validation report for one admitted stage execution result. |
| `load-checked-in-stage-reports` | `load_checked_in_stage_reports()` | Load all checked-in stage reports and validate campaign/interface alignment and unique stage ownership. |
| `derive-container-report-portfolio-index` | `derive_container_report_portfolio_index(repository_root)` | Require the permanent run index to match its source authority, then derive checksummed current and historical container report entries. |
| `render-container-report-portfolio-index` | `render_container_report_portfolio_index_json(index)`, `render_container_report_portfolio_index_markdown(index)` | Render the admitted container report portfolio deterministically for machine and paper-facing consumers. |
| `acquire-fastq-corpus` | `acquire_fastq_corpus(request)` | Resolve a reviewed corpus specification against exact ENA run metadata and atomically publish a checksum-pinned acquisition plan. |
| `derive-fastq-corpus-acquisition-plan` | `derive_fastq_corpus_acquisition_plan(spec_path, records, retrieved_at_utc)` | Deterministically replay acquisition-plan derivation from typed offline ENA records. |
| `materialize-fastq-corpus` | `materialize_fastq_corpus(request)` | Re-verify or transfer checksum-pinned ENA files, validate every gzip FASTQ record, make admitted bytes read-only, and atomically publish a source-addressed governed catalog snapshot. |
| `materialize-reference-bundle` | `materialize_checked_in_reference_bundle(request)` | Re-verify a checksum-pinned reference archive, normalize its FASTA, derive FAI and sequence-dictionary identities, and compose a governed catalog snapshot without overwriting prior material. |
| `publish-governed-data-catalog-registry` | `publish_governed_data_catalog_registry(request)` | Re-hash every explicitly declared catalog snapshot and publish or idempotently revalidate one source-addressed immutable registry. |
| `build-benchmark-data-index` | `build_benchmark_data_index(request)` | Verify a catalog registry, select each source-addressed current v2 snapshot, and re-hash its governed data closure. |
| `publish-benchmark-data-index` | `publish_benchmark_data_index(request)` | Publish one immutable registry-bound data index and human-readable report. |
| `admit-fastq-benchmark-corpus` | `admit_fastq_benchmark_corpus(study_root, data_index_path, corpus_id, expected_sample_count, expected_file_count)` | Verify exact data-index/catalog closure and re-hash every governed FASTQ file. |
| `admit-bam-benchmark-corpus` | `admit_bam_benchmark_corpus(study_root, data_index_path, corpus_id)` | Verify data-index/catalog agreement and re-hash one governed coordinate-sorted BAM and BAI. |
| `ensure-fastq-read-length-expectations` | `ensure_fastq_read_length_expectations(study_root, data_index_path, corpus_id, expected_sample_count, expected_file_count)` | Independently derive and immutably publish exact FASTQ read-length expectations. |
| `prepare-fastq-profile-read-lengths-material` | `prepare_fastq_profile_read_lengths_material(...)` | Prepare complete source-locked executable material for one planned run-2042 tool attempt. |
| `prepare-bam-mapping-summary-material` | `prepare_bam_mapping_summary_material(...)` | Prepare complete source-locked executable material for one planned run-2004 tool attempt. |
| `derive-stage-coverage` | `derive_stage_coverage(registry, catalog, reports, required_commit)` | Derive typed retained-stage definition and admitted-evidence coverage with exact catalog tool closure and current-source freshness. |
| `publish-checked-in-stage-coverage` | `publish_checked_in_stage_coverage(repository_root, mode)` | Deterministically write or check the checked-in JSON, Markdown, and generated schema publications. |
| `publish-checked-in-stage-tool-closure` | `publish_checked_in_benchmark_stage_tool_closure(repository_root, mode)` | Write or check exact supported catalog closure against domain bindings and permanent benchmark protocols. |
| `derive-stage-universe` | `derive_stage_universe(repository_root, portfolio)` | Derive every BAM, FASTQ, and VCF stage disposition and enforce exact benchmark run-reservation closure. |
| `publish-checked-in-stage-universe` | `publish_checked_in_stage_universe(repository_root, mode)` | Deterministically write or check the canonical stage registry, review index, and generated JSON Schema. |
| `plan-stage-campaign-attempts` | `plan_stage_campaign_attempts(registry, repository_commit, campaign_registry_sha256, run_id, campaign_id, attempt_id, observed_at_utc)` | Deterministically derive a source-locked plan with one validated lifecycle record per declared campaign tool. |
| `recover-stage-campaign-retry-lineage` | `recover_stage_campaign_retry_lineage(root, registry, run_id, campaign_id, tip_attempt_id)` | Recover one explicit complete terminal lineage per declared tool, tolerating only untouched planned siblings and rejecting detached execution evidence. |
| `plan-stage-campaign-retry-attempts` | `plan_stage_campaign_retry_attempts(registry, repository_commit, campaign_registry_sha256, prior_attempts, attempt_id, observed_at_utc)` | Derive linked retry origins from a complete terminal campaign lineage. |
| `verify-campaign-plan-source` | `verify_campaign_plan_source(plan, repository_commit, campaign_registry_sha256)` | Reject a campaign plan unless its source commit and declaration digest match the executing checkout exactly. |
| `verify-case-campaign-plan-source` | `verify_case_campaign_plan_source(plan, case_matrix)` | Reject a case-scoped campaign plan unless its exact matrix, source commit, and declaration digest agree. |
| `benchmark-execution-spec-schema` | `benchmark_execution_spec_schema()` | Derive the governed JSON Schema for immutable executable benchmark specifications. |
| `validate-benchmark-execution-spec` | `validate_execution_spec(spec)` | Reject incomplete source locks, mutable result roots, in-place retries, undeclared acceptance outputs, and inconsistent runtime policy. |
| `build-benchmark-execution-spec` | `build_execution_spec(registry, interfaces, catalog, plan, identity, image, material)` | Derive one executable attempt from aligned campaign, interface, catalog, source, image, and domain-material authorities, persisting an external-artifact admission digest when that delivery path is selected. |
| `build-benchmark-stage-plans` | `build_benchmark_stage_plans(spec, binding, catalog_tool, units)` | Derive the complete immutable repetition-by-unit plan matrix from canonical binding, container, path, parameter, and resource authorities. |
| `load-benchmark-execution-authorities` | `load_execution_authorities(repository_root, inventory_path, repository_commit, plan)` | Load and hash exact campaign, interface, catalog, and image-inventory authorities, rejecting source or closure drift. |
| `ensure-benchmark-execution-spec` | `ensure_execution_spec(benchmark_root, spec)` | Crash-safely persist one immutable executable specification beside its wholly planned attempt lifecycle. |
| `load-benchmark-execution-spec` | `load_execution_spec(benchmark_root, identity)` | Load and validate a persisted executable specification against its immutable attempt identity. |
| `resolve-benchmark-runtime` | `resolve_benchmark_runtime(platform, runtime_override)` | Select an explicit container runtime and reject local execution before benchmark allocation. |
| `ensure-stage-campaign-attempts` | `ensure_stage_campaign_attempts(root, plan)` | Validate source locks, then preflight and crash-safely initialize every immutable planned attempt in a campaign cycle. |
| `plan-benchmark-cases` | `plan_benchmark_cases(request)` | Select one permanent run, close its governed data and tool authorities, and immutably publish its generic case matrix. |
| `admit-benchmark-cases` | `admit_benchmark_cases(request)` | Validate normalized observations against their exact case matrix and publish an immutable admitted set. |
| `aggregate-benchmark-cases` | `aggregate_benchmark_cases(request)` | Validate admitted observations and immutably publish deterministic case aggregations. |
| `issue-benchmark-citation-record` | `issue_benchmark_citation_record(request)` | Derive an immutable paper citation from an admitted receipt, evidence book, exact replay, source catalog, report images, and case-matrix data snapshots. |
| `select-checked-in-benchmark-publication` | `select_checked_in_benchmark_publication(request)` | Verify receipt, book, citation, replay, and case definitions before crash-safe case-index and current-publication selection and complete run-index regeneration. |
| `benchmark-attempt-directory` | `benchmark_attempt_directory(root, identity)` | Derive the permanent `run-<id>-<domain>-<stage>/<tool>/attempts/<campaign>/<attempt>` storage path from validated identity. |
| `initialize-attempt-lifecycle` | `initialize_attempt_lifecycle(root, record)` | Delegate atomic publication of an immutable planned attempt record to `bijux-dna-runtime`. |
| `append-attempt-lifecycle-transition` | `append_attempt_lifecycle_transition(root, identity, transition)` | Delegate validation and no-clobber publication of one sequence-addressed transition to `bijux-dna-runtime`. |
| `load-attempt-lifecycle` | `load_attempt_lifecycle(root, identity)` | Delegate reconstruction and validation of immutable lifecycle evidence to `bijux-dna-runtime`. |
| `plan-attempt-migration` | `plan_attempt_migration(study_root, created_at_utc)` | Classify historical attempts by retained `run.kind` and produce a byte inventory bound plan. |
| `apply-attempt-migration` | `apply_attempt_migration(study_root, plan_path, ledger_path, updated_at_utc)` | Move planned attempts by same-filesystem rename while maintaining an interruption-safe ledger. |
| `verify-attempt-migration` | `verify_attempt_migration(study_root, plan_path, ledger_path)` | Recompute every current source or target file size and SHA-256 against the exact plan. |
| `rollback-attempt-migration` | `rollback_attempt_migration(study_root, plan_path, ledger_path, updated_at_utc)` | Verify and restore every applied attempt to its exact historical source path. |
| `plan-workspace-convergence` | `plan_benchmark_workspace_convergence(canonical_root, legacy_root)` | Hash both roots, move unique legacy entries toward the canonical root, quarantine only byte-identical duplicates, and retain all differing entries for manual review. |
| `apply-workspace-convergence` | `apply_benchmark_workspace_convergence(plan)` | Apply each planned move by verified same-filesystem rename without deleting legacy bytes. |
| `rollback-workspace-convergence` | `rollback_benchmark_workspace_convergence(plan)` | Re-hash and restore every applied canonical or quarantine move to its exact legacy source. |
| `rollback-workspace-layout-report` | `rollback_benchmark_workspace_layout_report(report)` | Treat an applied normalization report as the complete checksum-backed rollback receipt. |
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
- `data/evidence/benchmark-expectations/<catalog-sha256>/fastq-profile-read-lengths/manifest.json`
- `data/catalog/snapshots/<repository-commit>/<portfolio-id>.json`

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
