#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::implicit_hasher,
    clippy::must_use_candidate,
    clippy::new_without_default
)]

pub mod composer;
pub mod errors;
pub mod executor;
pub mod observer;
pub mod planner;
pub mod types;
pub mod validator;

pub use executor::{
    docker_logs, docker_rm, docker_stats_mb, docker_wait, docker_wait_timeout, parse_mem_to_mb,
    resolve_image_for_run, run_merge_container, run_merge_container_with_timeout,
    run_multiqc_container, run_multiqc_container_with_timeout, run_tool_container,
    run_tool_container_with_timeout, run_validate_container, run_validate_container_with_timeout,
    ExecutionOutput, MergeExecutionOutput,
};
pub use observer::{
    hash_file_sha256, input_fastq_stats, length_histogram, output_fastq_stats,
    parse_fastqvalidator_count, SeqkitMetrics,
};
pub use planner::{
    normalize_correct_tool_list, normalize_filter_tool_list, normalize_merge_tool_list,
    normalize_qc2_tool_list, normalize_screen_tool_list, normalize_stats_tool_list,
    normalize_trim_tool_list, normalize_umi_tool_list, normalize_validate_tool_list,
};
pub use types::{init_logging, StdoutLogger};
pub use validator::validate_execution_outputs;

pub use composer::bench;
pub use composer::image_qa;
pub use composer::paths::{
    bench_base_dir, bench_tools_dir, image_qa_base_dir, image_qa_jsonl_path, image_qa_sqlite_path,
};

pub use bijux_environment::api::ResolvedImage;
