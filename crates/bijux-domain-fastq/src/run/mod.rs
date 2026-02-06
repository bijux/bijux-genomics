pub mod corpus;

#[allow(unused_imports)]
pub use bijux_core::foundation::input_assessment::{
    assess_input_dir, discover_fastq_files, write_input_assessment, InputAssessmentV1,
};
pub use corpus::{bench_corpus, BenchCorpus, BenchCorpusId, BenchDataset};
