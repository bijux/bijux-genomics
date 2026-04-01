use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub struct BenchFastqProfileReadLengthsArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "auto",
        help = "Tool selection: auto | all | <csv>"
    )]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
    #[arg(long, help = "Allow experimental and silver-tier tools")]
    pub allow_experimental: bool,
    #[arg(long, default_value_t = 1)]
    pub replicates: u32,
    #[arg(long, default_value_t = 1)]
    pub jobs: u32,
    #[arg(long)]
    pub ci_bootstrap: Option<u32>,
    #[arg(long, help = "Set governed stage threads before per-job scaling")]
    pub threads: Option<u32>,
    #[arg(long)]
    pub histogram_bins: Option<u32>,
}

#[derive(Debug, Args)]
pub struct BenchFastqFilterArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "auto",
        help = "Tool selection: auto | all | <csv>"
    )]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
    #[arg(long, help = "Allow experimental and silver-tier tools")]
    pub allow_experimental: bool,
    #[arg(long, default_value_t = 1)]
    pub replicates: u32,
    #[arg(long, default_value_t = 1)]
    pub jobs: u32,
    #[arg(long)]
    pub ci_bootstrap: Option<u32>,
    #[arg(long)]
    pub threads: Option<u32>,
    #[arg(long)]
    pub max_n: Option<u32>,
    #[arg(long)]
    pub max_n_fraction: Option<f64>,
    #[arg(long)]
    pub max_n_count: Option<u32>,
    #[arg(long = "low-complexity-threshold")]
    pub low_complexity_threshold: Option<f64>,
    #[arg(long)]
    pub entropy_threshold: Option<f64>,
    #[arg(long)]
    pub kmer_ref: Option<PathBuf>,
    #[arg(long)]
    pub polyx_policy: Option<String>,
}

#[derive(Debug, Args)]
pub struct BenchFastqFilterLowComplexityArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "auto",
        help = "Tool selection: auto | all | <csv>"
    )]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
    #[arg(long, help = "Allow experimental and silver-tier tools")]
    pub allow_experimental: bool,
    #[arg(long, default_value_t = 1)]
    pub replicates: u32,
    #[arg(long, default_value_t = 1)]
    pub jobs: u32,
    #[arg(long)]
    pub ci_bootstrap: Option<u32>,
    #[arg(long)]
    pub entropy_threshold: Option<f64>,
    #[arg(long)]
    pub polyx_threshold: Option<u32>,
}

#[derive(Debug, Args)]
pub struct BenchFastqRemoveDuplicatesArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "auto",
        help = "Tool selection: auto | all | <csv>"
    )]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
    #[arg(long, help = "Set governed stage threads before per-job scaling")]
    pub threads: Option<u32>,
    #[arg(long)]
    pub dedup_mode: Option<String>,
    #[arg(long)]
    pub keep_order: Option<bool>,
    #[arg(long, help = "Allow experimental and silver-tier tools")]
    pub allow_experimental: bool,
    #[arg(long, default_value_t = 1)]
    pub replicates: u32,
    #[arg(long, default_value_t = 1)]
    pub jobs: u32,
    #[arg(long)]
    pub ci_bootstrap: Option<u32>,
}

#[derive(Debug, Args)]
pub struct BenchFastqCorrectArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "auto",
        help = "Tool selection: auto | all | <csv>"
    )]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
    #[arg(long, help = "Allow experimental and silver-tier tools")]
    pub allow_experimental: bool,
    #[arg(long, default_value_t = 1)]
    pub replicates: u32,
    #[arg(long, default_value_t = 1)]
    pub jobs: u32,
    #[arg(long)]
    pub ci_bootstrap: Option<u32>,
    #[arg(long, help = "Set governed stage threads before per-job scaling")]
    pub threads: Option<u32>,
    #[arg(long)]
    pub quality_encoding: Option<String>,
    #[arg(long)]
    pub kmer_size: Option<u32>,
    #[arg(long)]
    pub musket_kmer_budget: Option<u64>,
    #[arg(long)]
    pub genome_size: Option<u64>,
    #[arg(long)]
    pub max_memory_gb: Option<u32>,
    #[arg(long)]
    pub trusted_kmer_artifact: Option<PathBuf>,
    #[arg(long)]
    pub conservative_mode: Option<bool>,
}

#[derive(Debug, Args)]
pub struct BenchFastqQcPostArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "auto",
        help = "Tool selection: auto | all | <csv>"
    )]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
    #[arg(long, help = "Allow experimental and silver-tier tools")]
    pub allow_experimental: bool,
    #[arg(long, default_value_t = 1)]
    pub replicates: u32,
    #[arg(long, default_value_t = 1)]
    pub jobs: u32,
    #[arg(long)]
    pub ci_bootstrap: Option<u32>,
    #[arg(
        long,
        default_value = "auto",
        help = "Aggregation backend: auto | multiqc"
    )]
    pub aggregation_engine: Option<String>,
    #[arg(long)]
    pub aggregation_scope: Option<String>,
    #[arg(
        long,
        value_name = "PATH",
        help = "Governed QC artifact manifest consumed by fastq.report_qc; required because report_qc aggregates upstream QC outputs and does not regenerate them from raw FASTQ inputs"
    )]
    pub governed_qc_manifest: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct BenchFastqStatsArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long)]
    pub threads: Option<u32>,
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "auto",
        help = "Tool selection: auto | all | <csv>"
    )]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
    #[arg(long, help = "Allow experimental and silver-tier tools")]
    pub allow_experimental: bool,
    #[arg(long, default_value_t = 1)]
    pub replicates: u32,
    #[arg(long, default_value_t = 1)]
    pub jobs: u32,
    #[arg(long)]
    pub ci_bootstrap: Option<u32>,
}

#[derive(Debug, Args)]
pub struct BenchFastqProfileOverrepresentedArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, help = "Set governed stage threads before per-job scaling")]
    pub threads: Option<u32>,
    #[arg(
        long,
        help = "Maximum number of ranked sequences to retain in governed outputs"
    )]
    pub top_k: Option<u32>,
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "auto",
        help = "Tool selection: auto | all | <csv>"
    )]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
    #[arg(long, help = "Allow experimental and silver-tier tools")]
    pub allow_experimental: bool,
    #[arg(long, default_value_t = 1)]
    pub replicates: u32,
    #[arg(long, default_value_t = 1)]
    pub jobs: u32,
    #[arg(long)]
    pub ci_bootstrap: Option<u32>,
}
