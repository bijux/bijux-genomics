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
- `BenchmarkDecision`
- `BenchmarkStageCampaignRegistry`
- `BenchmarkStageCampaignSpec`
- `BenchmarkStageCoverage`
- `BenchmarkStageCoverageSummary`
- `BenchmarkStageCoverageRow`
- `BenchmarkStageDefinitionStatus`
- `BenchmarkStageExecutionStatus`
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
- `BenchmarkScientificProfile`
- `BenchmarkProfileAssignment`
- `BenchmarkComparisonMetricTolerance`
- `DecisionRationale`
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
- `BenchmarkAttemptTransition`
- `BenchmarkAttemptRecordV2`
- `BenchmarkCycleStatusV2`
- `BenchmarkCampaignPlan`
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
- `BENCHMARK_ATTEMPT_SCHEMA_V1`
- `BENCHMARK_ATTEMPT_SCHEMA_V2`
- `BENCHMARK_CYCLE_STATUS_SCHEMA_V1`
- `BENCHMARK_CYCLE_STATUS_SCHEMA_V2`
- `BENCHMARK_CAMPAIGN_PLAN_SCHEMA_V1`
- `BENCHMARK_EXECUTION_SPEC_SCHEMA_V1`
- `BENCHMARK_SCHEDULER_OBSERVATION_SCHEMA_V1`
- `BENCHMARK_STAGE_CAMPAIGNS_SCHEMA_V1`
- `BENCHMARK_STAGE_COVERAGE_SCHEMA_V1`
- `BENCHMARK_PORTFOLIO_SCHEMA_V1`
- `BENCHMARK_STAGE_INTERFACES_SCHEMA_V1`
- `BENCHMARK_PIPELINE_PARAMETERS_SCHEMA_V1`
- `BENCHMARK_STAGE_REPORT_SCHEMA_V1`

Statistics:

- `robust_stats`

Schema derivation:

- `benchmark_campaign_plan_schema`
- `benchmark_execution_spec_schema`
- `benchmark_scheduler_observation_schema`
- `validate_scheduler_observation`
- `validate_execution_spec`

Namespaces:

- `compare`
- `contract`

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
- Public validation behavior must be reflected in contract tests and the
  relevant docs.
- Public ordering of lists, reports, and rationale traces must remain
  deterministic for identical inputs.

## Verification

Run from the repository root when changing public exports:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test schemas --no-default-features
```
