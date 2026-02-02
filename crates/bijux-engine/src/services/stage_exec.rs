// split to keep module size manageable

include!("stage_exec/context.rs");
include!("stage_exec/fastqc.rs");
include!("stage_exec/banks.rs");
include!("stage_exec/stats.rs");
include!("stage_exec/metrics.rs");
include!("stage_exec/execution_run.rs");
include!("stage_exec/execution_post.rs");
include!("stage_exec/execution_qc_reports.rs");
include!("stage_exec/execution_reports.rs");
include!("stage_exec/execution_records.rs");
include!("stage_exec/execution_progress.rs");
include!("stage_exec/execute.rs");
include!("stage_exec/hashing.rs");
