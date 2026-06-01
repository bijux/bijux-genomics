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
`stage_api` is a curated compatibility surface for downstream stage planning. It reexports stage specs, stage registry lookup, default tool selection, allowed tool selection, `plan_stage`, `StagePlanRequest`, the governed local-ready `local_align_plan` helper for `bam.align`, the governed local-smoke `local_validate_smoke_plans` helper plus `LocalValidateSmokeCasePlan` for `bam.validate`, the governed local-smoke `local_qc_pre_smoke_plans` helper plus `LocalQcPreSmokeCasePlan` for `bam.qc_pre`, and the governed local-smoke `local_mapping_summary_smoke_plans` helper plus `LocalMappingSummarySmokeCasePlan` for `bam.mapping_summary`.

## Stability Rules
- Additions must be documented here and covered by boundary or contract tests.
- Changes to plan JSON, execution graph topology, explain payloads, or tool selection behavior require snapshot review.
- Runtime execution behavior does not belong in this API.
