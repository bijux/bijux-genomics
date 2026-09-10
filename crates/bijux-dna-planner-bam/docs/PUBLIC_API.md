# bijux-dna-planner-bam Public API

The public API is the planner surface exported from `src/lib.rs`.

## Public Modules
- `tool_adapters`

## Root Exports
- `BamPlanner`
- `BamPipelineInputs`
- `BamPlanConfig`
- `StagePlanRequest`
- `bam_workflow_template_catalog`
- `plan_stage`
- `plan_bam_to_bam__default__v1`
- `plan_bam_to_bam__adna_shotgun__v1`
- `plan_bam_to_bam__adna_capture__v1`
- `plan_bam_workflow_template`
- `pipeline_id_catalog`
- `report_stage_step`
- `stage_api`
- `PLANNER_VERSION`

## `stage_api`
`stage_api` is a curated compatibility surface for downstream stage planning. It reexports stage specs, stage registry lookup, default tool selection, allowed tool selection, `plan_stage`, `StagePlanRequest`, the governed local-ready `local_align_plan` helper for `bam.align`, the governed all-tool `local_align_output_contract_plans` helper for `bam.align` retained-output proof coverage, the governed local-ready `local_contamination_plan` helper for `bam.contamination`, the governed local-smoke `local_validate_smoke_plans` helper plus `LocalValidateSmokeCasePlan` for `bam.validate`, the governed local-smoke `local_qc_pre_smoke_plans` helper plus `LocalQcPreSmokeCasePlan` for `bam.qc_pre`, the governed local-smoke `local_mapping_summary_smoke_plans` helper plus `LocalMappingSummarySmokeCasePlan` for `bam.mapping_summary`, the governed local-smoke `local_filter_smoke_plans` helper plus `LocalFilterSmokeCasePlan` for `bam.filter`, the governed local-smoke `local_mapq_filter_smoke_plans` helper plus `LocalMapqFilterSmokeCasePlan` for `bam.mapq_filter`, the governed local-smoke `local_length_filter_smoke_plans` helper plus `LocalLengthFilterSmokeCasePlan` for `bam.length_filter`, the governed local-smoke `local_markdup_smoke_plans` helper plus `LocalMarkdupSmokeCasePlan` for `bam.markdup`, the governed local-smoke `local_duplication_metrics_smoke_plans` helper plus `LocalDuplicationMetricsSmokeCasePlan` for `bam.duplication_metrics`, the governed local-smoke `local_complexity_smoke_plans` helper plus `LocalComplexitySmokeCasePlan` for `bam.complexity`, the governed local-smoke `local_coverage_smoke_plans` helper plus `LocalCoverageSmokeCasePlan` for `bam.coverage`, the governed local-smoke `local_insert_size_smoke_plans` helper plus `LocalInsertSizeSmokeCasePlan` for `bam.insert_size`, the governed local-smoke `local_gc_bias_smoke_plans` helper plus `LocalGcBiasSmokeCasePlan` for `bam.gc_bias`, the governed local-smoke `local_endogenous_content_smoke_plans` helper plus `LocalEndogenousContentSmokeCasePlan` for `bam.endogenous_content`, the governed local-smoke `local_overlap_correction_smoke_plans` helper plus `LocalOverlapCorrectionSmokeCasePlan` for `bam.overlap_correction`, the governed local-smoke `local_damage_smoke_plans` helper plus `LocalDamageSmokeCasePlan` for `bam.damage`, the governed all-tool `local_damage_output_contract_plans` helper for `bam.damage` retained-output proof coverage, the governed local-smoke `local_authenticity_smoke_plans` helper plus `LocalAuthenticitySmokeCasePlan` for `bam.authenticity`, the governed all-tool `local_authenticity_output_contract_plans` helper for `bam.authenticity` retained-output proof coverage, the governed local-smoke `local_sex_smoke_plans` helper plus `LocalSexSmokeCasePlan` for `bam.sex`, and the governed all-tool `local_sex_output_contract_plans` helper for `bam.sex` retained-output proof coverage.

## Stability Rules
- Additions must be documented here and covered by boundary or contract tests.
- Changes to plan JSON, execution graph topology, explain payloads, or tool selection behavior require snapshot review.
- Runtime execution behavior does not belong in this API.

`stage_api::load_bam_domain_tool_definition` returns an immutable
`BamDomainToolDefinition` for one observed tool YAML. Its `contract_metadata`,
`planning_spec`, and `execution_spec` methods project that same observation.
Existing path-based loaders remain available and reload on each call. Definition
reuse is scoped by the caller; it does not cache changing repository files globally.

`stage_api::assess_bam_tool_definition` projects `BamToolDefinitionReadiness` from
an optional observed definition and a stage identity. Its typed contract and adapter
statuses preserve independent metadata admission and planning/execution-spec viability.
The stable status labels remain available through `as_str`. Runnable spec viability
does not assert tool installation, successful execution, or scientific qualification.

`stage_api::plan_stage_at(root, request)` resolves current BAM size and reference
sidecar observations relative to an explicit root. Logical input/output paths and
structured command arguments are preserved. The original `plan_stage` and direct
alignment adapter retain their caller-relative behavior. Planning does not change
the process working directory; missing size observations retain the existing zero
estimate, and sidecar presence alone does not establish scientific compatibility.

`assess_bam_output_declarations` owns report-artifact selection and declaration
closure. A recognized report must occur in both the stage output contract and
tool-specific expected artifacts. Missing outputs and duplicate declarations
are rejected or reported explicitly; stage defaults cannot invent a tool output.
The result describes declarations, not payload validity or scientific substitution.
