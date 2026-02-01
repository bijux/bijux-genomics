use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct BenchFastqTrimArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub ci_bootstrap: Option<u32>,
    pub adapter_bank_preset: Option<String>,
    pub adapter_bank: Option<String>,
    pub adapter_bank_file: Option<PathBuf>,
    pub enable_adapters: Vec<String>,
    pub disable_adapters: Vec<String>,
    pub polyx_preset: Option<String>,
    pub contaminant_preset: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqValidateArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub strict: bool,
    pub replicates: u32,
    pub ci_bootstrap: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqFilterArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub ci_bootstrap: Option<u32>,
    pub max_n: Option<u32>,
    pub low_complexity_threshold: Option<f64>,
    pub kmer_ref: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqMergeArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: PathBuf,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub ci_bootstrap: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqCorrectArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub ci_bootstrap: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqQcPostArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub ci_bootstrap: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqUmiArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub ci_bootstrap: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqScreenArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub ci_bootstrap: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqStatsArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub ci_bootstrap: Option<u32>,
}

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct BenchFastqPreprocessArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub out: PathBuf,
    pub strict: bool,
    pub auto: bool,
    pub objective: bijux_core::selection::Objective,
    pub bench_corpus: Option<crate::pipeline::BenchCorpusId>,
    pub allow_partial: bool,
    pub replicates: u32,
    pub ci_bootstrap: Option<u32>,
    pub adapter_bank_preset: Option<String>,
    pub adapter_bank: Option<String>,
    pub adapter_bank_file: Option<PathBuf>,
    pub enable_adapters: Vec<String>,
    pub disable_adapters: Vec<String>,
    pub polyx_preset: Option<String>,
    pub contaminant_preset: Option<String>,
    pub no_qc_post: bool,
    pub force_merge: bool,
}
