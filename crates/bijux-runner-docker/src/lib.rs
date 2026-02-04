//! Owner: bijux-runner-docker
//! Docker-backed execution and stage plugins.

pub mod executor;
pub mod exec_helpers;
pub mod observer;
pub mod replay;
pub mod stage_exec;
pub mod support;

pub mod primitives {
    pub use crate::executor::{
        docker_logs, docker_rm, docker_stats_mb, docker_wait, docker_wait_timeout, parse_mem_to_mb,
        resolve_image_for_run, run_filter_container, run_merge_container,
        run_merge_container_with_timeout, run_multiqc_container, run_multiqc_container_with_timeout,
        run_tool_container, run_tool_container_with_timeout, run_validate_container,
        run_validate_container_with_timeout, ExecutionOutput, MergeExecutionOutput,
    };
    pub use crate::exec_helpers::{
        cleanup_execution, execution_memory_mb, run_filter_execution, run_merge_execution,
        run_multiqc_execution, run_tool_execution, run_validate_execution,
    };
    pub use crate::observer::{
        hash_file_sha256, input_fastq_stats, length_histogram, output_fastq_stats,
        parse_fastqvalidator_count, write_explain_plan, Observer, SeqkitMetrics,
    };
    pub use crate::replay::replay_run;
    pub use crate::stage_exec::{execute_stage_plan, StageResultV1};
    pub use crate::support::build_tool_execution_spec;
}
