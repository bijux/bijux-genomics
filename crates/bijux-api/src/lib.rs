//! Public API entrypoints for planning and running bijux pipelines.

pub mod args;
pub mod bam_plan;
pub mod bam_router;
pub mod bam_support;
pub mod cross_router;
pub mod fastq_router;
pub mod fastq_stats_neutral;
pub mod run;

pub use args::{
    BamRunArgs, BenchBamPipelineArgs, BenchBamStageArgs, ExecuteRunRequest, ExecuteRunResult,
    FastqCrossArgs, PlanRunRequest, PlanRunResult, RenderReportRequest, RenderReportResult,
    RunRequest, RunResult,
};
pub use bijux_stages_fastq::args as fastq_args;
pub use run::{
    execute_run, plan_run, render_report, run_pipeline, select_pipeline, select_pipelines, RunMode,
};

pub use bijux_analyze::compare::compare_runs_with_baseline;
pub use bijux_analyze::export::write_stage_summary_csv;
pub use bijux_analyze::{
    build_rankings, compare_runs, load_facts_auto, load_run_summary, print_bench_schema,
    write_correct_report, write_filter_report, write_merge_report, write_qc_post_report,
    write_run_report_from_facts, write_run_summary_from_facts, write_stats_report,
    write_trim_report, write_umi_report, write_validate_report, RankInput,
};
pub use bijux_core::selection::{objective_spec, Objective};
pub use bijux_core::{
    load_manifests, load_profile, new_run_id, run_dir, DryRunExecutor, Executor, PathSpec, Profile,
    RunSpec, StageId, ToolId, ToolRegistry, ToolRole,
};
pub use bijux_core::{FactsRowV1, StagePlanV1};
pub use bijux_domain_bam::{bam_stage_completeness, BamStage};
pub use bijux_domain_fastq::banks as fastq_banks;
pub use bijux_domain_fastq::{
    adapter_bank_path, adapter_presets_path, benchmark_runs, contaminant_motifs_path,
    contaminant_presets_path, contaminant_references_dir, load_adapter_bank, load_adapter_presets,
    load_contaminant_motifs, load_contaminant_presets, load_polyx_bank, load_polyx_presets,
    polyx_bank_path, polyx_presets_path, qc_class_for_stage, write_benchmark_exports,
    AdapterPresetsV1, BenchCorpusId, EffectiveAdapterSet, QcClass, ReadScope, STAGES,
};
pub use bijux_engine::primitives::{
    build_tool_execution_spec, execute_stage_plan, init_logging, replay::replay_run,
    ExecutionManifest,
};
pub use bijux_env_builder::image_qa::run_image_qa;
pub use bijux_env_runtime::api::{
    available_runners, cache_dir, docker_image_exists, load_image_catalog, load_platform,
    resolve_image, PlatformSpec, RunnerKind, ToolImageSpec,
};
pub use bijux_infra::normalize_run_base_dir;
pub use bijux_pipelines::registry::PipelineRegistry;
pub use bijux_pipelines::{Domain, PipelineProfile};

pub mod run_index {
    pub use bijux_core::run_index::*;
}
