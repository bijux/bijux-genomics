# Public API

The public API is the planner surface exported from `src/lib.rs`.

## Public Modules
- `stage_api`
- `tool_adapters`

## Root Exports
- `FastqPlanner`
- `FastqPlanConfig`
- `FastqPipelineInputs`
- `FastqStageBenchmarkConfig`
- `FastqStageBinding`
- `FastqStageToolsetBinding`
- `FastqStageParameters`
- `StageArtifactInputBinding`
- `StageArtifactInputPolicy`
- `DefaultPipelineOptions`
- `PreprocessPolicyDecision`
- `CorrectDecisionTrace`
- `MergeDecisionTrace`
- `PreprocessDecisions`
- `PLANNER_VERSION`
- `TOOL_SEQKIT`
- `STAGE_PREPROCESS_SUMMARY`
- `STAGE_REPORT_AGGREGATE`
- `STAGE_COMPARE_STAGE_TOOLS`
- `STAGE_SELECT_STAGE_TOOL`
- `args`
- `plan_fastq_to_fastq__default__v1`
- `plan_fastq_to_bam__default__v1`
- `compose_fastq_stage_bindings`
- `compose_fastq_stage_bindings_with_dependencies`
- `scale_tool_spec_for_jobs`
- `default_pipeline_spec`
- `apply_preprocess_policy`
- `plan_preprocess`
- `preprocess_decisions`
- `resolve_preprocess_pipeline`
- `report_stage_step`
- `cross_fastq_to_bam_id_catalog`

## Stage API Exports
The `stage_api` module is the curated compatibility surface for downstream stage planning, toolset selection, stage-tool maturity, benchmark cohorts, bank resolution helpers, and stage spec access.

- `StageToolCapability`
- `StageToolBenchmarkProfile`
- `BenchmarkCohort`
- `BenchmarkReadinessLevel`
- `StageToolMaturityLevel`
- `ToolsetExecutionMode`
- `StagePlanJson`
- `toolset_for_stage`
- `stage_tool_maturity`
- `benchmark_cohorts_for_stage`
- `local_correct_errors_smoke_plans`
- `local_extract_umis_smoke_plans`
- `local_deplete_host_plan`
- `local_deplete_reference_contaminants_plan`
- `local_deplete_rrna_plan`
- `local_detect_adapters_smoke_plans`
- `local_detect_duplicates_premerge_smoke_plans`
- `local_estimate_library_complexity_prealign_smoke_plans`
- `local_filter_low_complexity_smoke_plans`
- `local_filter_reads_smoke_plans`
- `local_infer_asvs_smoke_plans`
- `local_index_reference_plan`
- `local_merge_pairs_smoke_plans`
- `local_normalize_primers_smoke_plans`
- `local_profile_overrepresented_sequences_smoke_plans`
- `local_profile_read_lengths_smoke_plans`
- `local_profile_reads_smoke_plans`
- `local_remove_chimeras_smoke_plans`
- `local_remove_duplicates_smoke_plans`
- `local_trim_polyg_tails_smoke_plans`
- `local_trim_reads_smoke_plans`
- `local_trim_terminal_damage_smoke_plans`
- `local_validate_reads_smoke_plans`

## Stability Rules
- Additions must be documented here and covered by boundary or contract tests.
- Changes to plan JSON, graph topology, explain payloads, tool selection, or benchmark cohort behavior require snapshot review.
- Runtime execution and output parsing do not belong in this API.
