//! Stable, intended-for-use engine interfaces.

pub use crate::core::composer::{
    load_registry, normalize_correct_tool_list, normalize_filter_tool_list,
    normalize_merge_tool_list, normalize_qc2_tool_list, normalize_screen_tool_list,
    normalize_stats_tool_list, normalize_trim_tool_list, normalize_umi_tool_list,
    normalize_validate_tool_list,
};
pub use crate::core::types::{
    default_domain_registry, trace_enabled, Capability, DataArtifact, Dependency, DomainRegistry,
    ExecutionContext, ExecutionManifest, ExecutionPlan, ExplainExclusion, ExplainPlan, MetricSet,
    PipelineSpec, Policy, ReadSet, RunPlan, SequenceCollection, StageGraph, StageNode,
    StageRequirement, StageResult, ToolInvocation,
};
pub use crate::core::types::{init_logging, StdoutLogger};
pub use crate::core::validator::validate_execution_outputs;
pub use crate::services::composer::paths::{
    bench_base_dir, bench_tools_dir, image_qa_base_dir, image_qa_jsonl_path, image_qa_sqlite_path,
};
pub use crate::services::composer::replay;
pub use crate::services::executor::{
    docker_logs, docker_rm, docker_stats_mb, docker_wait, docker_wait_timeout, parse_mem_to_mb,
    resolve_image_for_run, run_merge_container, run_merge_container_with_timeout,
    run_multiqc_container, run_multiqc_container_with_timeout, run_tool_container,
    run_tool_container_with_timeout, run_validate_container, run_validate_container_with_timeout,
    ExecutionOutput, MergeExecutionOutput,
};
pub use crate::services::observer::{
    hash_file_sha256, input_fastq_stats, length_histogram, output_fastq_stats,
    parse_fastqvalidator_count, write_explain_plan, SeqkitMetrics,
};
pub use bijux_environment::api::ResolvedImage;
