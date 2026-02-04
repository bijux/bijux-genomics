// split to keep module size manageable

include!("stage_exec/context.rs");
include!("stage_exec/banks.rs");
include!("stage_exec/helpers/hashing.rs");
include!("stage_exec/execution/run.rs");
include!("stage_exec/execution/post.rs");
include!("stage_exec/execution/reports.rs");
include!("stage_exec/execution/progress.rs");
include!("stage_exec/execution/records.rs");
include!("stage_exec/execution/qc_reports.rs");
include!("stage_exec/execute.rs");
