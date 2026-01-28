//! FASTQ domain definitions and contracts.
//!
//! Owns: FASTQ stage semantics, invariants, contracts, and pipeline specifications.
//! Must NOT depend on: bijux-engine or runtime/container execution logic.
// Reading order:
// 1. domain.rs
// 2. core types
// 3. stage semantics
// 4. metrics spec
// 5. execution adapters
// Structural layout of this crate is frozen as of FASTQ v1.
mod adapter;
mod contract;
mod contracts;
mod domain;
mod invariants;
mod metrics;
mod pipeline;
mod stages;

pub use contract::{
    contract_for_stage as canonical_contract_for_stage, FastqStage, StageContract, StageIO,
};
pub use contracts::{
    contract_for_stage, ensure_umi_headers, infer_input_kind, inspect_headers, log_header_warnings,
    normalize_outputs, preflight_stage, qc_class_for_stage, FastqArtifact, FastqArtifactKind,
    FastqLayout, FastqPE, FastqPairedEnd, FastqSE, FastqSampleId, FastqSingleEnd,
    FastqStageContract, FastqStats, HeaderInspection, NormalizedOutputs, QcClass, RawFailure,
};
pub use metrics::deltas::{compute_delta, ratio_u64};
pub use pipeline::query::get_results;
pub use pipeline::{
    append_event, bench_corpus, benchmark_runs, canonical_pipeline, canonical_tool_defaults,
    create_run_layout, fastq_default_pipeline, fastq_minimal_pipeline, now_string,
    update_run_index, write_benchmark_exports, write_environment, write_input_assessment,
    write_manifest, write_run_metadata, BenchCorpus, BenchCorpusId, BenchDataset,
    DefaultPipelineOptions, InputAssessmentV1, RunEnvironment, RunIndexEntry, RunLayout,
    RunManifest, RunStageEntry, ToolImageDigest,
};
pub use pipeline::{assess_input_dir, discover_fastq_files};
pub use stages::args;
