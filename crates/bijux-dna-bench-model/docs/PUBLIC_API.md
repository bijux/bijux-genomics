# bijux-dna-bench-model Public API

The stable consumer surface is curated in `src/public_api/stable_surface.rs` and
then re-exported from `src/lib.rs`. Consumers should prefer the crate root for
stable names and use namespaces only when they need grouped operations such as
`contract::validate_suite` or `compare::compare_summaries`.

## Stable Root Exports

Model contracts:

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
- `BenchmarkStageDownstreamProbe`
- `BenchmarkPipelineParameterConfig`
- `BenchmarkStageParameterConfig`
- `BenchmarkParameterValue`
- `BenchmarkStageReport`
- `BenchmarkStageReportDesign`
- `BenchmarkToolReport`
- `BenchmarkMeasurementContract`
- `BenchmarkCorpusContract`
- `BenchmarkCorpusClass`
- `FastqCorpusSourceSpec`
- `FastqCorpusSourceSample`
- `BenchmarkScientificProfile`
- `BenchmarkProfileAssignment`
- `BenchmarkComparisonMetricTolerance`
- `BenchmarkLifecycleStatus`
- `BenchmarkJobReference`
- `BenchmarkSchedulerTerminalState`
- `BenchmarkSchedulerAccounting`
- `BenchmarkSchedulerObservedState`
- `BenchmarkSchedulerEvidenceSource`
- `BenchmarkSchedulerObservationDraft`
- `BenchmarkSchedulerObservation`
- `BenchmarkEvidenceReference`
- `BenchmarkRunIdentity`
- `BenchmarkAttemptIdentity`
- `BenchmarkRunDefinition`
- `BenchmarkRunIndex`
- `BenchmarkRunIndexEntry`
- `BenchmarkRunIndexPublication`
- `BenchmarkPublicationReceipt`
- `BenchmarkCitationRecord`
- `BenchmarkAttemptTransition`
- `BenchmarkAttemptRecordV2`
- `BenchmarkCycleStatusV2`
- `BenchmarkCampaignPlan`
- `BenchmarkCyclePreparationState`
- `BenchmarkCyclePreparationUnit`
- `BenchmarkCyclePreparation`
- `BenchmarkCycleAdmissionState`
- `BenchmarkCycleAdmissionUnit`
- `BenchmarkCycleAdmission`
- `BenchmarkExecutionSpec`
- `BenchmarkExecutionSourceLock`
- `BenchmarkExecutionImage`
- `BenchmarkExecutionInput`
- `BenchmarkExecutionCommands`
- `BenchmarkExecutionParameterEvidence`
- `BenchmarkExecutionInterface`
- `BenchmarkExecutionMeasurement`
- `BenchmarkExecutionOutputs`
- `BenchmarkExecutionResources`
- `BenchmarkExecutionRuntimePolicy`
- `BenchmarkExecutionAcceptance`
- `ActiveScopeFastValidationReport`
- `ActiveScopeFastValidationCheck`
- `AppraisalReport`
- `AppraisalFinding`
- `AppraisalSummary`
- `HardeningQueueReport`
- `HardeningQueueEntry`
- `CrossBenchmarkCampaignReport`
- `CrossBenchmarkCampaignSummary`
- `CrossBenchmarkGoalEntry`
- `CrossMatrixRowRef`
- `CrossScenario`
- `FastqBenchmarkCampaignReport`
- `BamBenchmarkCampaignReport`
- `VcfBenchmarkCampaignReport`
- `HardeningBenchmarkCampaignReport`
- `DomainBenchmarkCampaignSummary`
- `DomainBenchmarkGoalEntry`
- `DomainBenchmarkMatrixRowRef`
- `DomainBenchmarkScenario`
- `BenchmarkMatrixReport`
- `BenchmarkMatrixSummary`
- `BenchmarkMatrixRow`
- `BenchmarkSurfaceMatch`
- `BenchmarkReadiness`
- `ContainerReportPortfolioIndex`
- `ContainerReportFreshness`
- `ContainerIndexedReport`
- `CorpusAcquisitionPlan`
- `CorpusAcquisitionSample`
- `CorpusAcquisitionFile`
- `CorpusAcquisitionLayout`
- `CorpusSourcePolicy`
- `GovernedDataCatalog`
- `GovernedDataCatalogSample`
- `GovernedDataCatalogFile`
- `GovernedDataCatalogRegistry`
- `GovernedDataCatalogRegistryRecord`
- `GovernedDataCatalogSnapshot`
- `GovernedDataCatalogSnapshotPublication`
- `GovernedDataCatalogRegistryPublication`
- `BenchmarkCampaignReportPublication`
- `CorpusMaterializationDisposition`
- `BenchmarkAttemptRecord`
- `BenchmarkCycleStatus`
- `BenchmarkGraphNode`
- `BenchmarkGraphNodeKind`
- `BenchmarkStageEdge`
- `BenchmarkStageSpec`
- `BenchmarkParamBinding`
- `DatasetSpec`
- `AnalysisRequirements`
- `DiversityRequirements`
- `ReplicatePolicy`
- `StratificationRequirement`
- `MetricsEnvelope`
- `MetricSummary`
- `SummaryRow`
- `SummaryStratum`

Policy and diagnostics:

- `GatePolicy`
- `GatePolicyOverrides`
- `GateDecision`
- `GateViolation`
- `BenchError`
- `SUITE_SCHEMA_V1`
- `CORPUS_SCHEMA_V1`
- `BUNDLE_SCHEMA_V1`
- `OBSERVATION_SCHEMA_V1`
- `SUMMARY_SCHEMA_V1`
- `DECISION_SCHEMA_V1`
- `BENCHMARK_ATTEMPT_SCHEMA_V1`
- `BENCHMARK_ATTEMPT_SCHEMA_V2`
- `BENCHMARK_CYCLE_STATUS_SCHEMA_V1`
- `BENCHMARK_CYCLE_STATUS_SCHEMA_V2`
- `BENCHMARK_CAMPAIGN_PLAN_SCHEMA_V1`
- `BENCHMARK_CAMPAIGN_REPORT_PUBLICATION_SCHEMA_V1`
- `BENCHMARK_EXECUTION_SPEC_SCHEMA_V1`
- `BENCHMARK_SCHEDULER_OBSERVATION_SCHEMA_V1`
- `BENCHMARK_STAGE_CAMPAIGNS_SCHEMA_V1`
- `BENCHMARK_STAGE_COVERAGE_SCHEMA_V1`
- `BENCHMARK_STAGE_UNIVERSE_SCHEMA_V1`
- `BENCHMARK_PORTFOLIO_SCHEMA_V1`
- `BENCHMARK_PORTFOLIO_SCHEMA_V2`
- `BENCHMARK_SCIENTIFIC_READINESS_SCHEMA_V1`
- `BENCHMARK_SCIENTIFIC_READINESS_SCHEMA_V2`
- `BENCHMARK_DATA_PORTFOLIO_AUDIT_SCHEMA_V1`
- `BENCHMARK_DATA_PORTFOLIO_AUDIT_SCHEMA_V2`
- `BENCHMARK_STAGE_INTERFACES_SCHEMA_V1`
- `BENCHMARK_PIPELINE_PARAMETERS_SCHEMA_V1`
- `BENCHMARK_STAGE_REPORT_SCHEMA_V1`
- `ACTIVE_SCOPE_FAST_VALIDATION_SCHEMA_V1`
- `APPRAISAL_REPORT_SCHEMA_V1`
- `HARDENING_QUEUE_REPORT_SCHEMA_V1`
- `CROSS_BENCHMARK_CAMPAIGN_SCHEMA_V1`
- `BENCHMARK_MATRIX_REPORT_SCHEMA_V1`
- `CORPUS_ACQUISITION_SCHEMA_V1`
- `CORPUS_ACQUISITION_SCHEMA_V2`
- `FASTQ_CORPUS_SOURCE_SCHEMA_V1`
- `GOVERNED_DATA_CATALOG_SCHEMA_V1`
- `GOVERNED_DATA_CATALOG_SNAPSHOT_PUBLICATION_SCHEMA_V1`
- `GOVERNED_DATA_CATALOG_REGISTRY_PUBLICATION_SCHEMA_V1`
- `GOVERNED_DATA_CATALOG_SCHEMA_V2`
- `GOVERNED_DATA_CATALOG_REGISTRY_SCHEMA_V1`
- `BENCHMARK_DATA_INDEX_SCHEMA_V1`
- `BENCHMARK_DATA_INDEX_SCHEMA_V2`

Statistics:

- `robust_stats`

Identity operations:

- `benchmark_attempt_identity_sha256`
- `benchmark_run_definition_sha256`
- `validate_benchmark_run_definition_revision`

Schema derivation:

- `benchmark_attempt_identity_schema`
- `benchmark_campaign_plan_schema`
- `benchmark_suite_spec_schema`
- `benchmark_corpus_manifest_schema`
- `benchmark_bundle_manifest_schema`
- `benchmark_observation_schema`
- `benchmark_summary_schema`
- `gate_decision_schema`
- `benchmark_execution_spec_schema`
- `benchmark_scheduler_observation_schema`
- `active_scope_fast_validation_report_schema`
- `appraisal_report_schema`
- `hardening_queue_report_schema`
- `cross_benchmark_campaign_report_schema`
- `benchmark_matrix_report_schema`
- `corpus_acquisition_plan_schema`
- `fastq_corpus_source_spec_schema`
- `governed_data_catalog_schema`
- `governed_data_catalog_registry_schema`
- `benchmark_data_index_schema`
- `validate_scheduler_observation`
- `validate_active_scope_fast_validation_report`
- `validate_appraisal_report`
- `validate_hardening_queue_report`
- `validate_cross_benchmark_campaign_report`
- `validate_benchmark_matrix_report`
- `validate_execution_spec`
- `validate_corpus_acquisition_plan`
- `validate_fastq_corpus_source_spec`
- `validate_governed_data_catalog`
- `validate_governed_data_catalog_registry`
- `validate_benchmark_data_index`
- `validate_utc_timestamp`

Normalized BAM execution metrics:

- `normalized_result::bam_envelope::bam_normalized_metrics_stage_contracts`
- `normalized_result::bam_envelope::bam_normalized_metrics_contract_for_stage`
- `normalized_result::bam_envelope::validate_bam_normalized_metrics`
- `normalized_result::bam_envelope::render_bam_normalized_metrics_schema`
- `normalized_result::bam_envelope::BAM_NORMALIZED_METRICS_SCHEMA_VERSION`
- `normalized_result::bam_envelope::BAM_NORMALIZED_METRICS_SCHEMA_ID`

Namespaces:

- `compare`
- `contract`
- `normalized_result`

## Managed Operations

The callable pure-model operations are listed in `docs/COMMANDS.md`. Public API
docs must not invent a separate command list; the command document is the SSOT
for operation names and entrypoints.

## Compatibility Rules

- Additions must preserve serialization compatibility or include an explicit
  breaking-change review under `docs/CHANGE_RULES.md`.
- New public exports must be added through `src/public_api/stable_surface.rs` so
  schema tests can detect the change.
- V2 attempt transitions are append-only and retain checksummed evidence plus
  terminal scheduler accounting. A retry receives a new permanent attempt
  identity and records explicit same-run, same-tool supersession.
- Run-protocol changes increase canonical `protocol_version` under the same run
  identity or allocate a replacement run with explicit supersession.
- Public validation behavior must be reflected in contract tests and the
  relevant docs.
- Public ordering of lists, reports, and rationale traces must remain
  deterministic for identical inputs.

## Verification

Run from the repository root when changing public exports:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test schemas --no-default-features
```
