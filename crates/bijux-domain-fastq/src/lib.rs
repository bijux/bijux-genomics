// Reading order:
// 1. domain.rs
// 2. core types
// 3. stage semantics
// 4. metrics spec
// 5. execution adapters
// Structural layout of this crate is frozen as of FASTQ v1.
mod adapter;
mod analyze;
mod contracts;
mod domain;
mod invariants;
mod metrics;
mod pipeline;
mod stages;

pub use analyze::report::{
    print_bench_schema, write_correct_report, write_filter_report, write_merge_report,
    write_qc_post_report, write_stats_report, write_trim_report, write_umi_report,
    write_validate_report,
};
pub use contracts::{
    contract_for_stage, ensure_umi_headers, infer_input_kind, inspect_headers, log_header_warnings,
    normalize_outputs, preflight_stage, qc_class_for_stage, FastqArtifact, FastqArtifactKind,
    FastqLayout, FastqPE, FastqPairedEnd, FastqSE, FastqSampleId, FastqSingleEnd,
    FastqStageContract, FastqStats, HeaderInspection, NormalizedOutputs, QcClass, RawFailure,
};
pub use pipeline::query::get_results;
pub use pipeline::{
    bench_corpus, benchmark_runs, canonical_pipeline, canonical_tool_defaults, create_run_layout,
    fastq_default_pipeline, write_benchmark_exports, BenchCorpus, BenchCorpusId, BenchDataset,
    DefaultPipelineOptions, InputAssessmentV1,
};
pub use stages::*;
