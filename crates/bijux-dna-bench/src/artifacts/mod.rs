//! Owner: bijux-dna-bench
//! Benchmark source inspection, output admission, and deterministic persistence.

pub mod admission;
mod fastq_score_evidence;
pub use fastq_score_evidence::load_fastq_tool_score_evidence;
mod source_assets;
pub mod writer;

pub use source_assets::{
    inspect_benchmark_source_asset, BenchmarkSourceAsset, BenchmarkSourceKind,
};

pub use admission::{
    admit_benchmark_execution_outputs, revalidate_benchmark_execution_outputs,
    BenchmarkExecutionAdmission, BenchmarkExecutionAdmissionPublication, BenchmarkMetricAssessment,
    BenchmarkOutputFileEvidence, BENCHMARK_EXECUTION_ADMISSION_SCHEMA_V1,
};
pub use writer::{
    archive_effective_tool_parameters, read_observations_jsonl, write_decision_json,
    write_decisions_json, write_observations_jsonl, write_summary_json,
    EffectiveToolParametersPublication, WriteMode, EFFECTIVE_TOOL_PARAMETERS_FILE,
};

pub(crate) mod bam_score_evidence;

pub(crate) mod vcf_score_evidence;

pub(crate) mod vcf_execution_proof;

pub(crate) mod schema_publication;

pub(crate) mod simulation;

pub(crate) mod command_artifacts;

pub(crate) mod micro_report;
