# Public API

`bijux-dna-bench` exposes a curated stable surface from the crate root:

```rust
pub use public_api::*;
```

The `public_api` module itself is private. Stable exports are owned by
`src/public_api/stable_surface.rs` and re-exported through `src/public_api/mod.rs`.

## Stable Operations

- `load_suite(path)` loads and validates a benchmark suite TOML file.
- `load_stage_campaign_registry(path)` loads and validates a benchmark stage-campaign registry TOML file.
- `load_benchmark_portfolio(path)` loads and validates a benchmark portfolio TOML file.
- `load_stage_interface_catalog(path)` loads and validates the stage-interface catalog, parameter configuration, and referenced repository contracts.
- `load_stage_interface_catalog_from_repo(path, repository_root)` performs the same admission against an explicit workspace root for production callers.
- `load_stage_interface_catalog_for_publication(path, source_repository_root, presentation_repository_root)` keeps historical interface fields source-locked while completing only absent render metadata from an explicit presentation authority.
- `load_checked_in_benchmark_contracts()` preserves the campaign/portfolio return contract while validating every checked-in planning input.
- `load_checked_in_benchmark_planning_contracts()` returns the campaign registry, portfolio, and stage-interface catalog after validating defined identity consistency.
- `derive_benchmark_matrix(request)` derives every selected stage/tool case from the supported product catalog, retains externally delivered tools, and emits explicit refusal rows for missing stage contracts or bindings.
- `load_benchmark_matrix_report(path)` parses and admits a persisted matrix before appraisal or campaign use.
- `publish_benchmark_matrix(path, report)` validates summary and readiness closure before atomically publishing a matrix.
- `current_benchmark_matrix_timestamp()` supplies the explicit generation identity used by thin operator surfaces.
- `appraise_matrix_report(matrix)` runs every registered product appraiser against an admitted matrix and validates exact finding-summary closure.
- `hardening_queue_from_appraisal(report)` groups admitted findings into a deterministic, contiguous hardening queue.
- `load_appraisal_report(path)` and `load_hardening_queue_report(path)` parse and admit persisted appraisal products.
- `publish_appraisal_report(path, report)` and `publish_hardening_queue_report(path, report)` validate before atomic publication.
- `derive_cross_benchmark_campaign(matrix, appraisal, queue, requested_goals)` validates source identities and derives the selected cross-domain goal closure.
- `load_cross_benchmark_campaign_report(path)` parses and admits a persisted cross-domain campaign report.
- `publish_cross_benchmark_campaign_report(path, report)` validates before atomic publication.
- `derive_bam_benchmark_campaign(matrix, appraisal, queue, requested_goals)` validates source identities and derives the selected BAM goal closure.
- `load_bam_benchmark_campaign_report(path)` parses and admits a persisted BAM campaign report.
- `publish_bam_benchmark_campaign_report(path, report)` validates before atomic publication.
- `derive_fastq_benchmark_campaign(matrix, appraisal, queue, requested_goals)` validates source identities and derives the selected FASTQ goal closure.
- `load_fastq_benchmark_campaign_report(path)` parses and admits a persisted FASTQ campaign report.
- `publish_fastq_benchmark_campaign_report(path, report)` validates before atomic publication.
- `derive_vcf_benchmark_campaign(matrix, appraisal, queue, requested_goals)` validates source identities and derives the selected VCF goal closure.
- `load_vcf_benchmark_campaign_report(path)` parses and admits a persisted VCF campaign report.
- `publish_vcf_benchmark_campaign_report(path, report)` validates before atomic publication.
- `derive_hardening_benchmark_campaign(matrix, appraisal, queue, requested_goals)` validates source identities and derives the selected portfolio hardening closure.
- `load_hardening_benchmark_campaign_report(path)` parses and admits a persisted hardening campaign report.
- `publish_hardening_benchmark_campaign_report(path, report)` validates before atomic publication.
- `derive_stage_universe(repository_root, portfolio)` derives the complete domain-stage registry and enforces exact benchmark run-reservation closure.
- `publish_checked_in_stage_universe(repository_root, mode)` writes or checks the generated canonical registry, review index, and JSON Schema.
- `load_stage_report(path)` returns a typed admission envelope binding the exact source bytes and canonical parsed report to separate SHA-256 identities.
- `load_stage_reports_from_dir(path, registry, interfaces)` admits checksummed reports against supplied authorities; raw report values are never returned to aggregation callers.
- `load_checked_in_stage_reports()` admits all checked-in reports against campaign and interface authority.
- `load_benchmark_stage_result_manifest(path)` parses and structurally validates one persisted stage execution result.
- `validate_benchmark_stage_result_path(repository_root, path)` derives its stable repository-relative validation report.
- `validate_benchmark_local_summary(report)` admits the persisted local readiness rollup before publication.
- `validate_benchmark_local_stage_manifest_completion(report)` admits manifest-presence evidence before publication.
- `validate_benchmark_local_stage_output_completion(report)` admits declared-output completeness evidence before publication.
- `validate_benchmark_local_stage_inventory(inventory)` admits one domain-local stage inventory.
- `validate_benchmark_local_all_domain_stage_inventory(report)` admits exact cross-domain inventory and count closure.
- `validate_benchmark_local_corpus_skip_report(report)` admits corpus replacement lineage and complete stage dispositions.
- `validate_benchmark_local_hpc_job_graph(graph)` admits exact job identity, dependency, root, ordering, and counter closure.
- `validate_fastq_comparable_metrics_report(report)`, `validate_bam_comparable_metrics_report(report)`, and `validate_vcf_comparable_metrics_report(report)` admit domain-specific comparable-metric readiness reports before a thin renderer persists them.
- `validate_stage_centric_report(report)`, `validate_tool_centric_report(report)`, and `validate_corpus_centric_report(report)` admit complementary persisted readiness views with complete nested and top-level aggregate closure.
- `validate_stage_tool_benchmark_ready_report(report)` admits the stage/tool readiness envelope only when pair identities, surface evidence, counters, and gate state close exactly.
- `validate_all_domain_active_stage_tool_matrix(report)`, `validate_all_domain_retained_tools(report)`, and `validate_stage_registry_extra_pairs(report)` admit the persisted stage/tool scope, retained product roster, and registry-drift views while retaining all supported licensed bindings.
- `derive_container_report_portfolio_index(repository_root)` derives the current and historical container report portfolio while requiring the permanent run index to match its source authority.
- `render_container_report_portfolio_index_json(index)` and `render_container_report_portfolio_index_markdown(index)` render the admitted portfolio deterministically.
- `publish_benchmark_campaign_report_with_presentation_authority(source_repository_root, presentation_repository_root, study_root, benchmark_root, plan, output_root)` publishes a source-locked historical campaign and returns the exact presentation commit, catalog digest, and overlaid stage IDs in its receipt.
- `publish_case_benchmark_campaign_report_with_presentation_authority(source_repository_root, presentation_repository_root, study_root, benchmark_root, plan, case_matrix, output_root)` preserves the exact verified case-matrix authority through generic dispatch and stage-owned publication.
- `acquire_fastq_corpus(request)` resolves a reviewed corpus specification against exact ENA run metadata and atomically publishes a no-clobber acquisition plan.
- `derive_fastq_corpus_acquisition_plan(spec_path, records, retrieved_at_utc)` deterministically derives the same validated plan from typed offline ENA records.
- `materialize_fastq_corpus(request)` re-verifies or transfers checksum-pinned ENA files, validates every gzip FASTQ record, makes the admitted files read-only, and atomically publishes a no-clobber governed catalog.
- `admit_fastq_benchmark_corpus(study_root, data_index_path, corpus_id, expected_sample_count, expected_file_count)` re-hashes and admits one exact governed FASTQ closure.
- `admit_bam_benchmark_corpus(study_root, data_index_path, corpus_id)` re-hashes and admits one exact governed coordinate-sorted BAM and BAI closure.
- `ensure_fastq_read_length_expectations(study_root, data_index_path, corpus_id, expected_sample_count, expected_file_count)` independently derives and immutably publishes exact read-length evidence.
- `prepare_fastq_profile_read_lengths_material(...)` prepares the complete run-2042 attempt input, command, parameter, output, resource, and acceptance closure.
- `prepare_bam_mapping_summary_material(...)` prepares the complete run-2004 BAM mapping-summary attempt closure.
- `plan_stage_campaign_attempts(registry, repository_commit, campaign_registry_sha256, run_id, campaign_id, attempt_id, observed_at_utc)` deterministically derives a source-locked plan with one validated attempt per declared tool.
- `recover_stage_campaign_retry_lineage(root, registry, run_id, campaign_id, tip_attempt_id)` reconstructs the explicit terminal lineage for every declared tool while rejecting detached execution evidence.
- `plan_stage_campaign_retry_attempts(registry, repository_commit, campaign_registry_sha256, prior_attempts, attempt_id, observed_at_utc)` derives linked retry origins from one complete terminal lineage.
- `verify_campaign_plan_source(plan, repository_commit, campaign_registry_sha256)` rejects stale source or campaign declaration identity before execution state is created.
- `ensure_stage_campaign_attempts(root, plan)` validates source locks, then preflights and crash-safely initializes every immutable planned attempt in a campaign cycle.
- `resolve_benchmark_runtime(platform, runtime_override)` applies the benchmark-owned container-runtime policy before execution is allocated.
- `plan_benchmark_cases(request)` selects one permanent run, closes its governed data and tool bindings, and immutably publishes its generic case matrix.
- `BenchmarkStagePluginRegistry` validates complete, non-ambiguous plugin
  coverage and generically materializes or normalizes a binding-backed stage.
- `BenchmarkStageNormalization` returns the normalized plugin output together
  with its stage-owned downstream evaluation without converting scientific
  failure into control-flow failure.
- `build_benchmark_stage_plans(spec, binding, catalog_tool, units)` constructs the complete
  immutable measured-repetition by execution-unit plan matrix while rejecting identity, path,
  parameter, and resource drift.
- `admit_benchmark_cases(request)` validates normalized observations against their exact case matrix and publishes an immutable admitted set.
- `aggregate_benchmark_cases(request)` validates an admitted observation set and publishes deterministic case aggregations.
- `benchmark_attempt_directory(root, identity)` derives the permanent attempt path from validated identity.
- `initialize_attempt_lifecycle(root, record)` delegates publication of one immutable planned attempt to runtime.
- `append_attempt_lifecycle_transition(root, identity, transition)` delegates one no-clobber lifecycle event to runtime.
- `load_attempt_lifecycle(root, identity)` delegates reconstruction and validation of the complete event chain to runtime.
- `summarize(suite, observations, options)` builds a deterministic
  `BenchmarkSummary`.
- `compare(summary_a, summary_b)` compares completed summaries.
- `gate(policy, summary)` evaluates summary rows against a `GatePolicy`.
- `bench_data_dir()` resolves the crate-owned benchmark data directory.
- `bench_suites_dir()` resolves the crate-owned suite catalog directory.
- `bench_contracts_dir()` resolves the repository-owned benchmark contract directory.
- `bench_stage_campaigns_path()` resolves the repository-owned stage-campaign registry path.
- `bench_portfolio_path()` resolves the repository-owned benchmark portfolio path.
- `bench_stage_interfaces_path()` resolves the repository-owned stage-interface catalog path.
- `bench_reports_dir()` resolves the repository-owned benchmark report directory.
- `bench_local_config_dir()` resolves the repository-owned local benchmark config directory.
- `bench_fastq_local_stage_matrix_path()` resolves the governed FASTQ local stage matrix path.
- `bench_bam_local_stage_matrix_path()` resolves the governed BAM local stage matrix path.

`docs/COMMANDS.md` is the SSOT for this callable operation list.

## Stable Types

The public API re-exports benchmark model contracts from `bijux-dna-bench-model`,
including:

- `AllDomainParserCollectorSourceKind`
- `AllDomainParserCollectorRow`
- `AllDomainParserCollectorReport`
- `BenchmarkSuiteSpec`
- `BenchmarkObservation`
- `BenchmarkSummary`
- `BenchmarkStageCampaignRegistry`
- `BenchmarkStageCampaignSpec`
- `BenchmarkStageCoverage`
- `BenchmarkStageCoverageSummary`
- `BenchmarkStageCoverageRow`
- `BenchmarkStageDefinitionStatus`
- `BenchmarkStageExecutionStatus`
- `BenchmarkDisposition`
- `BenchmarkStageUniverse`
- `BenchmarkStageUniverseSummary`
- `BenchmarkStageUniverseRow`
- `BenchmarkPortfolio`
- `BenchmarkStageInterfaceCatalog`
- `BenchmarkStageInterfaceSpec`
- `BenchmarkPipelineParameterConfig`
- `BenchmarkMatrixRequest`
- `BenchmarkMatrixReport`
- `BenchmarkMatrixRow`
- `BenchmarkMatrixSummary`
- `BenchmarkStageReport`
- `BenchmarkStageResultManifest`
- `BenchmarkStageResultValidationReport`
- `BenchmarkLocalStageRuntimeMetricsReport`
- `BenchmarkLocalStageManifestCompletionReport`
- `BenchmarkLocalStageOutputCompletionReport`
- `BenchmarkLocalStageInventory`
- `BenchmarkLocalAllDomainStageInventory`
- `BenchmarkLocalCorpusSkipReport`
- `BenchmarkLocalHpcJobGraph`
- `BenchmarkToolReport`
- `BenchmarkRunIdentity`
- `BenchmarkAttemptIdentity`
- `BenchmarkAttemptRecordV2`
- `BenchmarkAttemptTransition`
- `BenchmarkCycleStatusV2`
- `BenchmarkCampaignPlan`
- `BenchmarkExecutionSpec`
- `BenchmarkStagePlanManifest`
- `BenchmarkStagePlanReference`
- `CorpusMaterializationRequest`
- `CorpusMaterializationPublication`
- `CorpusAcquisitionPlan`
- `GovernedDataCatalog`
- `AdmittedBamCorpus`
- `BamMappingSummaryMaterial`
- `ContainerReportPortfolioIndex`
- `ContainerReportFreshness`
- `FastqReadLengthExpectations`
- `FastqReadLengthExpectationPublication`
- `FastqProfileReadLengthsMaterial`
- `ComparableMetricContractStatus`
- `ComparableMetricDirection`
- `FastqComparableMetricsReport`
- `BamComparableMetricsReport`
- `VcfComparableMetricsReport`
- `StageCentricReport`
- `StageToolBenchmarkReadyReport`
- `ToolCentricReport`
- `CorpusCentricReport`
- `ReadinessComparisonContractStatus`
- `AllDomainActiveStageToolMatrixReport`
- `AllDomainRetainedToolsReport`
- `StageRegistryExtraPairsReport`
- `BenchmarkLocalVcfSmokeGoalContract`
- `BenchmarkLocalVcfSmokeGoalCheck`
- `BenchmarkLocalVcfSmokeSuiteReport`
- `GatePolicy`
- `GatePolicyOverrides`
- `GateDecision`
- `GateViolation`
- `MetricSummary`
- `SummaryRow`
- `BenchError`
- `BenchRunOptions`
- `benchmark_execution_spec_schema`
- `benchmark_stage_plan_manifest_schema`
- `benchmark_suite_spec_schema`
- `benchmark_corpus_manifest_schema`
- `benchmark_bundle_manifest_schema`
- `benchmark_local_corpus_skip_report_schema`
- `benchmark_observation_schema`
- `benchmark_summary_schema`
- `gate_decision_schema`
- `validate_execution_spec`
- `validate_stage_plan_manifest`
- `build_execution_spec`
- `BenchmarkExecutionMaterial`
- `BenchmarkStagePlanInputMaterial`
- `BenchmarkStagePlanOutputMaterial`
- `BenchmarkStagePlanUnitMaterial`
- `load_execution_authorities`
- `BenchmarkExecutionAuthorities`
- `ensure_execution_spec`
- `ensure_stage_plan_manifest`
- `load_stage_plan_manifest`

## Export Policy

- Add stable exports in `src/public_api/stable_surface.rs`.
- Keep `src/lib.rs` thin; do not scatter stable exports across the root.
- Do not expose raw `serde_json::Value` in the public API.
- Do not expose planner, runner, API, or domain execution types as benchmark
  operations.
- Update `docs/COMMANDS.md`, `docs/BENCH_CONTRACT.md`, and tests when adding a
  new public benchmark operation or artifact shape.
