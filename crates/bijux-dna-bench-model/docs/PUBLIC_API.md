# bijux-dna-bench-model Public API

The stable consumer surface is curated in `src/public_api/stable_surface.rs` and
then re-exported from `src/lib.rs`. Consumers should prefer the crate root for
stable names and use namespaces only when they need grouped operations such as
`contract::validate_suite` or `compare::compare_summaries`.

## Stable Root Exports

Model contracts:

- `BenchmarkLocalPipelineDagConfig`
- `BenchmarkLocalPipelineDagDomain`
- `BenchmarkLocalPipelineDagMetadataConfig`
- `BenchmarkLocalPipelineDagStagesConfig`
- `BenchmarkLocalPipelineDagNode`
- `BenchmarkLocalPipelineProjectSourceConfig`
- `BenchmarkSlurmSubmissionIntent`
- `BenchmarkSlurmSubmissionMaterial`
- `BenchmarkSlurmArraySpec`
- `BenchmarkSlurmDependencyCondition`
- `AllDomainParserCollectorSourceKind`
- `AllDomainParserCollectorRow`
- `AllDomainParserCollectorReport`
- `BenchmarkSuiteSpec`
- `BenchmarkObservation`
- `BenchmarkSummary`
- `BenchmarkStageCampaignRegistry`
- `BenchmarkCampaignMaterialDriver`
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
- `BenchmarkStageResultManifest`
- `BenchmarkStageResultValidationReport`
- `BenchmarkLocalStageRuntimeMetricsReport`
- `BenchmarkLocalStageRuntimeMetric`
- `BenchmarkLocalStageReadinessKind`
- `BenchmarkLocalStageInventory`
- `BenchmarkLocalStageInventoryEntry`
- `BenchmarkLocalAllDomainStageInventory`
- `BenchmarkLocalCorpusSkipReport`
- `BenchmarkLocalCorpusFixture`
- `BenchmarkLocalCorpusSkip`
- `BenchmarkLocalAssetBackedStage`
- `BenchmarkLocalPlannerOnlyStage`
- `BenchmarkLocalHpcJobGraph`
- `BenchmarkLocalHpcJobGraphNode`
- `BenchmarkLocalStageManifestCompletionReport`
- `BenchmarkLocalStageManifestCompletionEntry`
- `BenchmarkLocalStageOutputCompletionReport`
- `BenchmarkLocalStageOutputCompletionEntry`
- `BenchmarkLocalStageMissingOutputEntry`
- `BenchmarkLocalSummaryReport`
- `BenchmarkLocalSummaryStage`
- `BenchmarkLocalReadinessStatus`
- `derive_benchmark_local_readiness_status`
- `BenchmarkMeasurementContract`
- `BenchmarkCorpusContract`
- `BenchmarkCorpusClass`
- `FastqCorpusSourceSpec`
- `FastqCorpusSourceSample`
- `ComparableMetricContractStatus`
- `ComparableMetricDirection`
- `FastqComparableMetricsReport`
- `FastqComparableMetricsRow`
- `BamComparableMetricsReport`
- `BamComparableMetricsRow`
- `VcfComparableMetricsReport`
- `VcfComparableMetricsRow`
- `StageCentricReport`
- `StageCentricStageReport`
- `StageCentricToolRow`
- `StageToolBenchmarkReadyReport`
- `StageToolBenchmarkReadySurfaceSummary`
- `StageToolBenchmarkReadySurfaceStatus`
- `StageToolBenchmarkReadyFailingPair`
- `StageToolBenchmarkReadyExcludedPair`
- `ToolCentricReport`
- `ToolCentricToolReport`
- `ToolCentricStageRow`
- `CorpusCentricReport`
- `CorpusCentricCorpusReport`
- `CorpusCentricStageRow`
- `ReadinessComparisonContractStatus`
- `AllDomainActiveStageToolMatrixReport`
- `AllDomainActiveStageToolMatrixRow`
- `AllDomainRetainedToolsReport`
- `AllDomainRetainedToolRow`
- `StageRegistryExtraPairsReport`
- `StageRegistryExtraPairRow`
- `BenchmarkLocalVcfSmokeGoalContract`
- `BenchmarkLocalVcfSmokeGoalCheck`
- `BenchmarkLocalVcfSmokeSuiteReport`
- `BenchmarkLocalVcfCallGlSmokeMetrics`
- `BenchmarkLocalVcfCallGlSmokeReport`
- `BenchmarkLocalVcfCallSmokeMetrics`
- `BenchmarkLocalVcfCallSmokeReport`
- `BenchmarkLocalVcfIbdSmokeRow`
- `BenchmarkLocalVcfIbdInsufficientOverlapProbe`
- `BenchmarkLocalVcfIbdSmokeReport`
- `BenchmarkImputationQualityFieldAvailability`
- `BenchmarkLocalVcfImputationMetricsSmokeReport`
- `LocalVcfStageCatalogConfig`
- `VcfStageCatalogRow`
- `LocalVcfStageCatalogReport`
- `LocalVcfStageMatrixConfig`
- `VcfStageMatrixRow`
- `LocalVcfStageMatrixReport`
- `ValidateVcfStageMatrixReport`
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
- `BenchmarkProgressLedger`
- `BenchmarkProgressLedgerSource`
- `BenchmarkProgressFrontier`
- `BenchmarkProgressTask`
- `BenchmarkProgressTaskStatus`
- `BenchmarkProgressExecutionUnit`
- `BenchmarkProgressExecutionUnitStatus`
- `BenchmarkProgressOwner`
- `BenchmarkProgressAttempt`
- `BenchmarkProgressAttemptStatus`
- `BenchmarkProgressCommit`
- `BenchmarkProgressCheck`
- `BenchmarkProgressCheckStatus`
- `BenchmarkProgressExternalWait`
- `BenchmarkExecutionSpec`
- `BenchmarkStagePlanManifest`
- `BenchmarkStagePlanReference`
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
- `BENCHMARK_PROGRESS_LEDGER_SCHEMA_V1`
- `BENCHMARK_CAMPAIGN_PLAN_SCHEMA_V1`
- `BENCHMARK_CAMPAIGN_REPORT_PUBLICATION_SCHEMA_V1`
- `BENCHMARK_EXECUTION_SPEC_SCHEMA_V1`
- `BENCHMARK_STAGE_PLAN_MANIFEST_SCHEMA_V1`
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
- `BENCHMARK_STAGE_RESULT_SCHEMA_V2`
- `BENCHMARK_STAGE_RESULT_VALIDATION_SCHEMA_V1`
- `BENCHMARK_LOCAL_RUNTIME_METRICS_SCHEMA_V1`
- `BENCHMARK_LOCAL_STAGE_INVENTORY_SCHEMA_V1`
- `BENCHMARK_LOCAL_ALL_DOMAIN_STAGE_INVENTORY_SCHEMA_V1`
- `BENCHMARK_LOCAL_CORPUS_SKIP_REPORT_SCHEMA_V1`
- `BENCHMARK_LOCAL_HPC_JOB_GRAPH_SCHEMA_V1`
- `BENCHMARK_LOCAL_STAGE_MANIFEST_COMPLETION_SCHEMA_V1`
- `BENCHMARK_LOCAL_STAGE_OUTPUT_COMPLETION_SCHEMA_V1`
- `BENCHMARK_LOCAL_SUMMARY_SCHEMA_V1`
- `ACTIVE_SCOPE_FAST_VALIDATION_SCHEMA_V1`
- `APPRAISAL_REPORT_SCHEMA_V1`
- `HARDENING_QUEUE_REPORT_SCHEMA_V1`
- `CROSS_BENCHMARK_CAMPAIGN_SCHEMA_V1`
- `BENCHMARK_MATRIX_REPORT_SCHEMA_V1`
- `CORPUS_ACQUISITION_SCHEMA_V1`
- `CORPUS_ACQUISITION_SCHEMA_V2`
- `FASTQ_CORPUS_SOURCE_SCHEMA_V1`
- `FASTQ_COMPARABLE_METRICS_SCHEMA_V1`
- `BAM_COMPARABLE_METRICS_SCHEMA_V1`
- `VCF_COMPARABLE_METRICS_SCHEMA_V1`
- `STAGE_CENTRIC_REPORT_SCHEMA_V1`
- `TOOL_CENTRIC_REPORT_SCHEMA_V1`
- `CORPUS_CENTRIC_REPORT_SCHEMA_V1`
- `ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_SCHEMA_V1`
- `ALL_DOMAIN_RETAINED_TOOLS_SCHEMA_V1`
- `STAGE_REGISTRY_EXTRA_PAIRS_SCHEMA_V1`
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
- `benchmark_stage_plan_manifest_schema`
- `benchmark_slurm_submission_intent_schema`
- `benchmark_scheduler_observation_schema`
- `benchmark_local_hpc_job_graph_schema`
- `benchmark_local_corpus_skip_report_schema`
- `active_scope_fast_validation_report_schema`
- `appraisal_report_schema`
- `hardening_queue_report_schema`
- `cross_benchmark_campaign_report_schema`
- `benchmark_matrix_report_schema`
- `corpus_acquisition_plan_schema`
- `fastq_corpus_source_spec_schema`
- `fastq_comparable_metrics_schema`
- `bam_comparable_metrics_schema`
- `vcf_comparable_metrics_schema`
- `stage_centric_report_schema`
- `tool_centric_report_schema`
- `corpus_centric_report_schema`
- `all_domain_parser_collector_schema`
- `all_domain_active_stage_tool_matrix_schema`
- `all_domain_retained_tools_schema`
- `stage_registry_extra_pairs_schema`
- `benchmark_local_vcf_smoke_suite_schema`
- `benchmark_local_vcf_call_gl_smoke_metrics_schema`
- `benchmark_local_vcf_call_gl_smoke_schema`
- `benchmark_local_vcf_call_smoke_metrics_schema`
- `benchmark_local_vcf_call_smoke_schema`
- `benchmark_local_vcf_ibd_smoke_schema`
- `benchmark_local_vcf_imputation_metrics_smoke_schema`
- `governed_data_catalog_schema`
- `governed_data_catalog_registry_schema`
- `benchmark_data_index_schema`
- `validate_scheduler_observation`
- `validate_benchmark_local_hpc_job_graph`
- `validate_benchmark_local_corpus_skip_report`
- `validate_active_scope_fast_validation_report`
- `validate_appraisal_report`
- `validate_hardening_queue_report`
- `validate_cross_benchmark_campaign_report`
- `validate_benchmark_matrix_report`
- `validate_execution_spec`
- `validate_benchmark_slurm_submission_intent`
- `validate_corpus_acquisition_plan`
- `validate_fastq_corpus_source_spec`
- `validate_fastq_comparable_metrics_report`
- `validate_bam_comparable_metrics_report`
- `validate_vcf_comparable_metrics_report`
- `validate_stage_centric_report`
- `validate_tool_centric_report`
- `validate_corpus_centric_report`
- `validate_all_domain_parser_collector`
- `validate_all_domain_active_stage_tool_matrix`
- `validate_all_domain_retained_tools`
- `validate_stage_registry_extra_pairs`
- `validate_benchmark_local_vcf_smoke_suite`
- `validate_benchmark_local_vcf_call_gl_smoke_metrics`
- `validate_benchmark_local_vcf_call_gl_smoke_report`
- `validate_benchmark_local_vcf_call_smoke_metrics`
- `validate_benchmark_local_vcf_call_smoke_report`
- `validate_benchmark_local_vcf_ibd_smoke_report`
- `validate_benchmark_local_vcf_imputation_metrics_smoke_report`
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

Normalized FASTQ execution metrics:

- `normalized_result::fastq_envelope::fastq_normalized_metrics_stage_contracts`
- `normalized_result::fastq_envelope::fastq_normalized_metrics_contract_for_stage`
- `normalized_result::fastq_envelope::required_metrics_keys`
- `normalized_result::fastq_envelope::validate_fastq_normalized_metrics`
- `normalized_result::fastq_envelope::render_fastq_normalized_metrics_schema`
- `normalized_result::fastq_envelope::FASTQ_NORMALIZED_METRICS_SCHEMA_VERSION`
- `normalized_result::fastq_envelope::FASTQ_NORMALIZED_METRICS_SCHEMA_ID`

Normalized VCF execution metrics:

- `normalized_result::vcf_envelope::vcf_normalized_metrics_stage_descriptors`
- `normalized_result::vcf_envelope::validate_vcf_normalized_metrics`
- `normalized_result::vcf_envelope::render_vcf_normalized_metrics_schema`
- `normalized_result::vcf_envelope::render_vcf_normalized_metrics_stage_schema`
- `normalized_result::vcf_envelope::VcfNormalizedMetricsStageSchemaDescriptor`
- `normalized_result::vcf_envelope::VCF_NORMALIZED_METRICS_SCHEMA_ID`

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

## Local Pipeline Configuration

`validate_benchmark_local_pipeline_dag_config` admits the version and required identity/node
envelope of `bijux.bench.local_pipeline_dag.v1`. The model owns both the combined TOML
configuration shape and the separate YAML metadata/stages shapes, preserving strict
unknown-field handling and the project-source retry default of three.

`benchmark_local_pipeline_dag_config_schema`, `benchmark_local_pipeline_dag_metadata_config_schema`,
and `benchmark_local_pipeline_dag_stages_config_schema` produce their registered input schemas.
Existing validation-report schemas retain their persisted identities. Graph topology, stage
readiness, reference resolution and scientific profile validation are additional service
obligations; schema admission does not certify executable or scientifically compatible inputs.

`BenchmarkLocalStageMatrixConfig` owns the existing FASTQ/BAM local matrix input shape, with generated `benchmark_local_stage_matrix_config_schema()` and the two exact v1 schema identities. VCF retains `LocalVcfStageMatrixConfig`. Structural deserialization rejects unknown fields and readiness values; repository admission additionally checks the requested domain, exact version, non-empty inventory and unique stage identities. Serialization preserves matrix declaration order.

`derive_hardening_benchmark_campaign(matrix, appraisal, queue, requested_goals)` admits source identities and derives the existing hardening report entirely from typed inputs. Catalog scenarios describe benchmark readiness; this operation does not execute or certify a benchmark.

`BenchmarkSourceKind` and `BenchmarkSourceAsset` describe pre-staging content identities independently of filesystem acquisition. The benchmark repository owner observes current files, directories and index-prefix members. Existing variant spellings and fields remain unchanged; these in-memory observations are not a new persisted wire schema.

Runtime declaration classification uses `RuntimeResolutionInput` and returns `RuntimeResolutionSignature`. HPC selection consumes a borrowed `HpcApptainerTarget` and returns `HpcResolution`. Existing resolution vocabulary and unavailable diagnostics are preserved; these in-memory contracts introduce no persisted schema or runtime execution capability.

`BenchmarkRuntimeProbe` is the typed runtime-readiness projection of a domain tool declaration. Its entrypoint, container identity, version/help command and required-field accessors preserve existing interpretation. It intentionally does not validate unrelated full domain fields; repository acquisition separately binds the declared tool identity to the request.

Execution-mode declarations own closed TOML-compatible shapes and header/assignment admission. Unknown schemas, unsupported required fields and ambiguous tool assignments fail before executable resolution. This policy does not establish actual tool availability.

`render_hpc_submission_script` projects one campaign attempt from planned jobs and explicit foundation lock references. Scheduler directives precede executable statements; metadata is quoted as literal shell arguments. Header control characters and malformed dependency identities are rejected before script publication. Rendering performs no IO and does not certify stage execution.

`select_hpc_redaction_needles` applies campaign sensitive-key policy to explicit decoded values. `redact_hpc_text` shares literal replacement semantics with bundle string redaction; environment acquisition belongs to the benchmark repository reader.

`SlurmSubmissionReport` and `SubmittedJob` own the existing scheduler receipt wire format. `validate_hpc_submission_report` admits unique job identities and ordered in-scope dependencies; unsupported versions and modes reject during decoding. `HpcSubmissionSettings` and `HpcSubmissionMode` carry scope choices without command-parser dependencies. The report records submission, not successful execution.

`validate_hpc_submission_request` admits selected campaign identities and output fields before submission effects. `validate_hpc_scheduler_receipt` applies the report identity rules immediately after a backend response and before operator or encrypted artifact publication; duplicate receipts reject. These checks do not roll back a scheduler side effect that already occurred.

Slurm script-body evidence is owned by `BenchmarkLocalSlurmScriptBodyReport` and `BenchmarkLocalSlurmScriptBodyEntry`. `inspect_benchmark_local_slurm_script_body` applies the lexical placeholder/command-marker policy; `build_benchmark_local_slurm_script_body_report` constructs and admits aggregate evidence; `validate_benchmark_local_slurm_script_body_report` checks persisted versions, paths, uniqueness, findings and verdicts. The registered closed schema preserves the existing JSON keys and version. Text inspection is not shell parsing or proof of scientific execution.

`evaluate_benchmark_local_hpc_slurm_readiness` consumes a borrowed `BenchmarkLocalHpcSlurmObservation` and returns one stable local-readiness check. Full script, syntax, manifest and dependency report admission precedes a successful check; absent prerequisites remain failed checks. The retained goal IDs and valid-observation presentation remain unchanged. No scheduler or filesystem capability is part of this policy.

Slurm dependency inspection interprets borrowed submission jobs and observed script text, then constructs an admitted dependency-source diagnostic. It does not authorize scheduler submission or replace complete manifest/DAG validation.

`evaluate_benchmark_local_hpc_harness_readiness` admits borrowed fake-failure, output-completion, manifest-completion and runtime-metric observations before returning stable local HPC readiness checks. Synthetic evidence remains synthetic; a counter with missing or inconsistent rows cannot establish readiness.

`project_full_benchmark_observation_rows` consumes borrowed `FullBenchmarkSourceObservation` values for expected results, simulation, smoke subsets and failure classifications. Complete existing source admission precedes projection; source kinds, identifiers, missingness and evidence paths remain distinct. Final collector closure and independent scientific execution provenance still require their owning checks.

Failure simulation owns closed `AllDomainFakeFailureRecord` and `AllDomainFakeFailuresManifest` v1 wire contracts, `AllDomainFakeFailureOutputEntry`, their schema functions and version/root constants. `validate_all_domain_failure_simulation_record` and `validate_all_domain_failure_simulation_manifest` reject schema, identity, output-role/path, count and outcome contradictions. These artifacts describe illustrative failures and do not attest tool execution.

Missing-result audit observations use closed `AllDomainMissingResultTestReport`, `AllDomainMissingResultRow` and `AllDomainMissingResultStatus` contracts. `AllDomainMissingResultObservation` borrows aligned declarations, simulation rows and an optional typed stage manifest; `project_all_domain_missing_result_observation` preserves source identity and missingness without I/O. `admit_all_domain_missing_result_audit` computes the verdict after complete declared-content checks; `validate_all_domain_missing_result_audit` admits replay. The schema function, version/default-path constants and governed removal identities belong to the model. These are illustrative manifest-presence checks, not fresh scientific execution evidence.

The `completion` namespace owns closed all-domain completion reports, schema/version
identity, and pure admission. Admission checks every row, required-file identity,
manifest/runtime flag, artifact count, failure reason, aggregate counter, and the
five governed mutation identities before computing the behavior verdict. Replay
validation requires a consistent successful declared verdict; it does not prove
current filesystem state or fresh scientific execution.

`harness::validate_all_domain_harness_alignment` borrows the current typed reports
and checks exact simulation result coverage and benchmark-ready binding agreement.
It rejects blank, padded and repeated result identities before set comparison;
real-smoke parser records use a separate identity space. This policy performs no
IO and does not certify scientific payloads or external execution.

`project_all_domain_simulation_parser_row` admits illustrative metrics against a
result report and stage manifest before producing the four-field normalized
parser snapshot. Schema, six identities, counts, metric-name uniqueness, finite
duration, stage/tool/runtime and exact declared output observations must agree.
This proves report consistency; it does not verify scientific output contents.

`validate_hpc_campaign_config_source` parses declared TOML in memory before
confidentiality admission. Quoted and escaped keys, inline tables, nested values
and arrays obey the same policy as ordinary declarations. Non-empty confidential
Slurm fields and sensitive values reject; empty values, recognized placeholders
and declared secret-reference fields remain admissible. Diagnostics identify
keys without reproducing values; malformed TOML refuses without source excerpts.
This operation reads no environment or files and does not certify tool availability.
