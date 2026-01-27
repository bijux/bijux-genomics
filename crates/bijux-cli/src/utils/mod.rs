#![allow(unused_imports)]

mod contract;
mod docker;
mod hash;
mod logging;
mod run_merge;
mod run_tool;
mod run_validate;
mod seqkit;

pub use contract::validate_execution_outputs;
pub use docker::{
    docker_logs, docker_rm, docker_stats_mb, docker_wait, docker_wait_timeout, parse_mem_to_mb,
};
pub use hash::{
    bench_base_dir, bench_tools_dir, hash_file_sha256, image_qa_base_dir, image_qa_jsonl_path,
    image_qa_sqlite_path, normalize_run_base_dir,
};
pub use logging::{init_logging, StdoutLogger};
pub use run_merge::{run_merge_container, run_merge_container_with_timeout, MergeExecutionOutput};
pub use run_tool::{run_tool_container, run_tool_container_with_timeout, ExecutionOutput};
pub use run_validate::{
    run_multiqc_container, run_multiqc_container_with_timeout, run_validate_container,
    run_validate_container_with_timeout,
};
pub use seqkit::{
    input_fastq_stats, length_histogram, output_fastq_stats, parse_fastqvalidator_count,
    SeqkitMetrics,
};

pub use bijux_environment::api::ResolvedImage;
