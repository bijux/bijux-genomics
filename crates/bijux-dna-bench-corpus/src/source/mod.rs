//! Typed corpus source declarations and semantic admission, independent of campaign execution.
mod classification;
mod model;
mod validation;
pub use classification::BenchmarkCorpusClass;
pub use model::{
    fastq_corpus_source_spec_schema, CorpusAcquisitionLayout, CorpusEvaluationDesign,
    CorpusSourcePolicy, FastqCorpusSourceSample, FastqCorpusSourceSpec,
};
pub use validation::{
    validate_corpus_evaluation_design, validate_fastq_corpus_source_spec, validate_source_policy,
    CorpusSourceError,
};
pub const FASTQ_CORPUS_SOURCE_SCHEMA_V1: &str = "bijux.benchmark.fastq_corpus_source.v1";
