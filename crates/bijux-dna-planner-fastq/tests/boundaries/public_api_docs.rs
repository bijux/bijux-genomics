use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn public_api_docs_match_curated_exports() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let docs =
        std::fs::read_to_string(root.join("docs/PUBLIC_API.md")).expect("read docs/PUBLIC_API.md");

    assert_eq!(
        markdown_list_after_heading(&docs, "## Public Modules"),
        entries(["stage_api", "tool_adapters"]),
        "public module docs must match the root module surface"
    );
    assert_eq!(
        markdown_list_after_heading(&docs, "## Root Exports"),
        entries([
            "FastqPlanner",
            "FastqPlanConfig",
            "FastqPipelineInputs",
            "FastqStageBenchmarkConfig",
            "FastqStageBinding",
            "FastqStageToolsetBinding",
            "FastqStageParameters",
            "StageArtifactInputBinding",
            "StageArtifactInputPolicy",
            "DefaultPipelineOptions",
            "PreprocessPolicyDecision",
            "CorrectDecisionTrace",
            "MergeDecisionTrace",
            "PreprocessDecisions",
            "PLANNER_VERSION",
            "TOOL_SEQKIT",
            "STAGE_PREPROCESS_SUMMARY",
            "STAGE_REPORT_AGGREGATE",
            "STAGE_COMPARE_STAGE_TOOLS",
            "STAGE_SELECT_STAGE_TOOL",
            "args",
            "plan_fastq_to_fastq__default__v1",
            "plan_fastq_to_bam__default__v1",
            "compose_fastq_stage_bindings",
            "compose_fastq_stage_bindings_with_dependencies",
            "scale_tool_spec_for_jobs",
            "default_pipeline_spec",
            "apply_preprocess_policy",
            "plan_preprocess",
            "preprocess_decisions",
            "resolve_preprocess_pipeline",
            "report_stage_step",
            "cross_fastq_to_bam_id_catalog",
        ]),
        "root export docs must match the curated planner surface"
    );
    assert_eq!(
        markdown_list_after_heading(&docs, "## Stage API Exports"),
        entries([
            "StageToolCapability",
            "StageToolBenchmarkProfile",
            "BenchmarkCohort",
            "BenchmarkReadinessLevel",
            "StageToolMaturityLevel",
            "ToolsetExecutionMode",
            "StagePlanJson",
            "toolset_for_stage",
            "stage_tool_maturity",
            "benchmark_cohorts_for_stage",
            "local_correct_errors_smoke_plans",
            "local_extract_umis_smoke_plans",
            "local_deplete_host_plan",
            "local_deplete_reference_contaminants_plan",
            "local_deplete_rrna_plan",
            "local_detect_adapters_smoke_plans",
            "local_detect_duplicates_premerge_smoke_plans",
            "local_estimate_library_complexity_prealign_smoke_plans",
            "local_filter_low_complexity_smoke_plans",
            "local_filter_reads_smoke_plans",
            "local_infer_asvs_smoke_plans",
            "local_index_reference_plan",
            "local_merge_pairs_smoke_plans",
            "local_normalize_primers_smoke_plans",
            "local_profile_overrepresented_sequences_smoke_plans",
            "local_profile_read_lengths_smoke_plans",
            "local_profile_reads_smoke_plans",
            "local_remove_chimeras_smoke_plans",
            "local_remove_duplicates_smoke_plans",
            "local_trim_polyg_tails_smoke_plans",
            "local_trim_reads_smoke_plans",
            "local_trim_terminal_damage_smoke_plans",
            "local_validate_reads_smoke_plans",
        ]),
        "stage_api docs must match the curated compatibility surface"
    );
}

#[test]
fn documented_root_exports_remain_compilable() {
    use anyhow::Result;
    use bijux_dna_core::contract::{ArtifactRef, ExecutionGraph, ExecutionStep, PipelineSpec};
    use bijux_dna_core::prelude::ToolExecutionSpecV1;
    use bijux_dna_planner_fastq::{
        CorrectDecisionTrace, DefaultPipelineOptions, FastqPipelineInputs, FastqPlanConfig,
        FastqPlanner, FastqStageBenchmarkConfig, FastqStageBinding, FastqStageParameters,
        FastqStageToolsetBinding, MergeDecisionTrace, PreprocessDecisions,
        PreprocessPolicyDecision, StageArtifactInputBinding, StageArtifactInputPolicy,
        PLANNER_VERSION, STAGE_COMPARE_STAGE_TOOLS, STAGE_PREPROCESS_SUMMARY,
        STAGE_REPORT_AGGREGATE, STAGE_SELECT_STAGE_TOOL, TOOL_SEQKIT,
    };

    let _: FastqPlanner = FastqPlanner;
    let _: &str = PLANNER_VERSION;
    let _: &str = TOOL_SEQKIT;
    let _ = STAGE_PREPROCESS_SUMMARY;
    let _ = STAGE_REPORT_AGGREGATE;
    let _ = STAGE_COMPARE_STAGE_TOOLS;
    let _ = STAGE_SELECT_STAGE_TOOL;
    let _: fn(&FastqPlanConfig) -> Result<ExecutionGraph> = FastqPlanner::plan;
    let _: fn(&FastqStageBenchmarkConfig) -> Result<ExecutionGraph> =
        FastqPlanner::plan_stage_benchmark_cohort;
    let _: fn(&FastqPipelineInputs, DefaultPipelineOptions) -> Result<ExecutionGraph> =
        bijux_dna_planner_fastq::plan_fastq_to_fastq__default__v1;
    let _: fn(
        Vec<bijux_dna_stage_contract::StagePlanV1>,
        bijux_dna_core::contract::PlanPolicy,
    ) -> Result<ExecutionGraph> = bijux_dna_planner_fastq::plan_fastq_to_bam__default__v1;
    let _: fn(DefaultPipelineOptions) -> PipelineSpec =
        bijux_dna_planner_fastq::default_pipeline_spec;
    let _: fn(&str) -> Vec<String> = bijux_dna_planner_fastq::cross_fastq_to_bam_id_catalog;
    let _: fn(&ToolExecutionSpecV1, usize) -> ToolExecutionSpecV1 =
        bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
    let _: fn(&Path, Vec<ArtifactRef>, Vec<ArtifactRef>) -> ExecutionStep =
        bijux_dna_planner_fastq::report_stage_step;

    let _: Option<FastqStageBinding> = None;
    let _: Option<FastqStageToolsetBinding> = None;
    let _: Option<FastqStageParameters> = None;
    let _: Option<StageArtifactInputBinding> = None;
    let _: Option<StageArtifactInputPolicy> = None;
    let _: Option<PreprocessPolicyDecision> = None;
    let _: Option<CorrectDecisionTrace> = None;
    let _: Option<MergeDecisionTrace> = None;
    let _: Option<PreprocessDecisions> = None;
    let _: Option<bijux_dna_planner_fastq::args::BenchFastqPreprocessArgs> = None;
}

#[test]
fn documented_stage_api_exports_remain_compilable() {
    use bijux_dna_core::ids::{StageId, ToolId};

    let _: Option<bijux_dna_planner_fastq::stage_api::StageToolCapability> = None;
    let _: Option<bijux_dna_planner_fastq::stage_api::StageToolBenchmarkProfile> = None;
    let _: Option<bijux_dna_planner_fastq::stage_api::BenchmarkCohort> = None;
    let _: Option<bijux_dna_planner_fastq::stage_api::BenchmarkReadinessLevel> = None;
    let _: Option<bijux_dna_planner_fastq::stage_api::StageToolMaturityLevel> = None;
    let _: Option<bijux_dna_planner_fastq::stage_api::ToolsetExecutionMode> = None;
    let _: Option<bijux_dna_planner_fastq::stage_api::StagePlanJson> = None;
    let _: fn(&StageId, bijux_dna_planner_fastq::stage_api::ToolsetExecutionMode) -> Vec<ToolId> =
        bijux_dna_planner_fastq::stage_api::toolset_for_stage;
    let _: fn(
        &StageId,
        &ToolId,
    ) -> Option<bijux_dna_planner_fastq::stage_api::StageToolMaturityLevel> =
        bijux_dna_planner_fastq::stage_api::stage_tool_maturity;
    let _: fn(&StageId) -> Vec<bijux_dna_planner_fastq::stage_api::BenchmarkCohort> =
        bijux_dna_planner_fastq::stage_api::benchmark_cohorts_for_stage;
    let _: fn(
        &Path,
    )
        -> anyhow::Result<Vec<bijux_dna_planner_fastq::LocalCorrectErrorsSmokeCasePlan>> =
        bijux_dna_planner_fastq::stage_api::local_correct_errors_smoke_plans;
    let _: fn(&Path) -> anyhow::Result<bijux_dna_stage_contract::StagePlanV1> =
        bijux_dna_planner_fastq::stage_api::local_deplete_host_plan;
    let _: fn(&Path) -> anyhow::Result<bijux_dna_stage_contract::StagePlanV1> =
        bijux_dna_planner_fastq::stage_api::local_deplete_reference_contaminants_plan;
    let _: fn(&Path) -> anyhow::Result<bijux_dna_stage_contract::StagePlanV1> =
        bijux_dna_planner_fastq::stage_api::local_deplete_rrna_plan;
    let _: fn(&Path) -> anyhow::Result<bijux_dna_stage_contract::StagePlanV1> =
        bijux_dna_planner_fastq::stage_api::local_index_reference_plan;
    let _: fn(
        &Path,
    )
        -> anyhow::Result<Vec<bijux_dna_planner_fastq::LocalDetectAdaptersSmokeCasePlan>> =
        bijux_dna_planner_fastq::stage_api::local_detect_adapters_smoke_plans;
    let _: fn(
        &Path,
    ) -> anyhow::Result<
        Vec<bijux_dna_planner_fastq::LocalDetectDuplicatesPremergeSmokeCasePlan>,
    > = bijux_dna_planner_fastq::stage_api::local_detect_duplicates_premerge_smoke_plans;
    let _: fn(
        &Path,
    ) -> anyhow::Result<
        Vec<bijux_dna_planner_fastq::LocalEstimateLibraryComplexityPrealignSmokeCasePlan>,
    > = bijux_dna_planner_fastq::stage_api::local_estimate_library_complexity_prealign_smoke_plans;
    let _: fn(
        &Path,
    ) -> anyhow::Result<
        Vec<bijux_dna_planner_fastq::LocalFilterLowComplexitySmokeCasePlan>,
    > = bijux_dna_planner_fastq::stage_api::local_filter_low_complexity_smoke_plans;
    let _: fn(&Path) -> anyhow::Result<Vec<bijux_dna_planner_fastq::LocalInferAsvsSmokeCasePlan>> =
        bijux_dna_planner_fastq::stage_api::local_infer_asvs_smoke_plans;
    let _: fn(
        &Path,
    ) -> anyhow::Result<Vec<bijux_dna_planner_fastq::LocalFilterReadsSmokeCasePlan>> =
        bijux_dna_planner_fastq::stage_api::local_filter_reads_smoke_plans;
    let _: fn(&Path) -> anyhow::Result<Vec<bijux_dna_planner_fastq::LocalMergePairsSmokeCasePlan>> =
        bijux_dna_planner_fastq::stage_api::local_merge_pairs_smoke_plans;
    let _: fn(
        &Path,
    )
        -> anyhow::Result<Vec<bijux_dna_planner_fastq::LocalNormalizePrimersSmokeCasePlan>> =
        bijux_dna_planner_fastq::stage_api::local_normalize_primers_smoke_plans;
    let _: fn(
        &Path,
    ) -> anyhow::Result<
        Vec<bijux_dna_planner_fastq::LocalProfileOverrepresentedSequencesSmokeCasePlan>,
    > = bijux_dna_planner_fastq::stage_api::local_profile_overrepresented_sequences_smoke_plans;
    let _: fn(
        &Path,
    ) -> anyhow::Result<
        Vec<bijux_dna_planner_fastq::LocalProfileReadLengthsSmokeCasePlan>,
    > = bijux_dna_planner_fastq::stage_api::local_profile_read_lengths_smoke_plans;
    let _: fn(
        &Path,
    ) -> anyhow::Result<Vec<bijux_dna_planner_fastq::LocalProfileReadsSmokeCasePlan>> =
        bijux_dna_planner_fastq::stage_api::local_profile_reads_smoke_plans;
    let _: fn(
        &Path,
    )
        -> anyhow::Result<Vec<bijux_dna_planner_fastq::LocalRemoveChimerasSmokeCasePlan>> =
        bijux_dna_planner_fastq::stage_api::local_remove_chimeras_smoke_plans;
    let _: fn(
        &Path,
    )
        -> anyhow::Result<Vec<bijux_dna_planner_fastq::LocalRemoveDuplicatesSmokeCasePlan>> =
        bijux_dna_planner_fastq::stage_api::local_remove_duplicates_smoke_plans;
    let _: fn(
        &Path,
    )
        -> anyhow::Result<Vec<bijux_dna_planner_fastq::LocalTrimPolygTailsSmokeCasePlan>> =
        bijux_dna_planner_fastq::stage_api::local_trim_polyg_tails_smoke_plans;
    let _: fn(&Path) -> anyhow::Result<Vec<bijux_dna_planner_fastq::LocalTrimReadsSmokeCasePlan>> =
        bijux_dna_planner_fastq::stage_api::local_trim_reads_smoke_plans;
    let _: fn(
        &Path,
    ) -> anyhow::Result<
        Vec<bijux_dna_planner_fastq::LocalTrimTerminalDamageSmokeCasePlan>,
    > = bijux_dna_planner_fastq::stage_api::local_trim_terminal_damage_smoke_plans;
    let _: fn(
        &Path,
    )
        -> anyhow::Result<Vec<bijux_dna_planner_fastq::LocalValidateReadsSmokeCasePlan>> =
        bijux_dna_planner_fastq::stage_api::local_validate_reads_smoke_plans;
}

fn markdown_list_after_heading(markdown: &str, heading: &str) -> BTreeSet<String> {
    let mut values = BTreeSet::new();
    let mut in_section = false;

    for line in markdown.lines() {
        if line == heading {
            in_section = true;
            continue;
        }
        if in_section && line.starts_with("## ") {
            break;
        }
        if !in_section {
            continue;
        }
        if let Some(item) = line.strip_prefix("- `").and_then(|line| line.strip_suffix('`')) {
            values.insert(item.to_string());
        }
    }

    values
}

fn entries<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(str::to_string).collect()
}
