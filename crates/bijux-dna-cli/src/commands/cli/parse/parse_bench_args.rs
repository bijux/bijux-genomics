
#[derive(Debug, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum BenchCommand {
    Run(BenchRunArgs),
    Status,
    Fastq {
        #[command(subcommand)]
        command: BenchFastqCommand,
    },
    Bam {
        #[command(subcommand)]
        command: BenchBamCommand,
    },
    Schema {
        stage: String,
    },
}

#[derive(Debug, Args)]
pub struct BenchRunArgs {
    #[arg(long)]
    pub suite: String,
    #[arg(long, default_value_t = false)]
    pub hpc: bool,
}

#[derive(Debug, Subcommand)]
pub enum BenchFastqCommand {
    Trim(BenchFastqTrimArgs),
    Validate(BenchFastqValidateArgs),
    Filter(BenchFastqFilterArgs),
    Merge(BenchFastqMergeArgs),
    Correct(BenchFastqCorrectArgs),
    #[command(name = "qc-post", alias = "qc2")]
    QcPost(BenchFastqQcPostArgs),
    Umi(BenchFastqUmiArgs),
    Screen(BenchFastqScreenArgs),
    Stats(BenchFastqStatsArgs),
    Preprocess(BenchFastqPreprocessArgs),
}

#[derive(Debug, Args)]
pub struct BenchFastqTrimArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, value_delimiter = ',', default_value = "auto", help = "Tool selection: auto | all | <csv>")]
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
    #[arg(long, help = "Adapter bank preset name (default: illumina-default)")]
    pub adapter_bank_preset: Option<String>,
    #[arg(
        long,
        help = "Adapter bank selection: preset:<name> (deprecated; use --adapter-bank-preset)"
    )]
    pub adapter_bank: Option<String>,
    #[arg(long, help = "Adapter bank file (yaml/json)")]
    pub adapter_bank_file: Option<PathBuf>,
    #[arg(long)]
    pub enable_adapter: Vec<String>,
    #[arg(long)]
    pub disable_adapter: Vec<String>,
    #[arg(long, help = "PolyX preset name (default: illumina_twocolor)")]
    pub polyx_preset: Option<String>,
    #[arg(long, help = "Contaminant preset name (default: illumina_default)")]
    pub contaminant_preset: Option<String>,
}

#[derive(Debug, Args)]
pub struct BenchFastqValidateArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, value_delimiter = ',', default_value = "auto", help = "Tool selection: auto | all | <csv>")]
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
    pub strict: bool,
}

#[derive(Debug, Args)]
pub struct BenchFastqFilterArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, value_delimiter = ',', default_value = "auto", help = "Tool selection: auto | all | <csv>")]
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
    pub max_n: Option<u32>,
    #[arg(long = "low-complexity-threshold")]
    pub low_complexity_threshold: Option<f64>,
    #[arg(long)]
    pub kmer_ref: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct BenchFastqMergeArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, value_delimiter = ',', default_value = "auto", help = "Tool selection: auto | all | <csv>")]
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
pub struct BenchFastqCorrectArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, value_delimiter = ',', default_value = "auto", help = "Tool selection: auto | all | <csv>")]
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
pub struct BenchFastqQcPostArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, value_delimiter = ',', default_value = "auto", help = "Tool selection: auto | all | <csv>")]
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
pub struct BenchFastqUmiArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, value_delimiter = ',', default_value = "auto", help = "Tool selection: auto | all | <csv>")]
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
pub struct BenchFastqScreenArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, value_delimiter = ',', default_value = "auto", help = "Tool selection: auto | all | <csv>")]
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
pub struct BenchFastqStatsArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, value_delimiter = ',', default_value = "auto", help = "Tool selection: auto | all | <csv>")]
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
#[allow(clippy::struct_excessive_bools)]
pub struct BenchFastqPreprocessArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long, help = "Pipeline profile id (default, minimal)")]
    pub pipeline_profile: Option<String>,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long)]
    pub strict: bool,
    #[arg(long, help = "Allow experimental and silver-tier tools")]
    pub allow_experimental: bool,
    #[arg(long, default_value_t = 1)]
    pub replicates: u32,
    #[arg(long, default_value_t = 1)]
    pub jobs: u32,
    #[arg(long)]
    pub ci_bootstrap: Option<u32>,
    #[arg(long, help = "Adapter bank preset name (default: illumina-default)")]
    pub adapter_bank_preset: Option<String>,
    #[arg(
        long,
        help = "Adapter bank selection: preset:<name> (deprecated; use --adapter-bank-preset)"
    )]
    pub adapter_bank: Option<String>,
    #[arg(long, help = "Adapter bank file (yaml/json)")]
    pub adapter_bank_file: Option<PathBuf>,
    #[arg(long)]
    pub enable_adapter: Vec<String>,
    #[arg(long)]
    pub disable_adapter: Vec<String>,
    #[arg(long, help = "PolyX preset name (default: illumina_twocolor)")]
    pub polyx_preset: Option<String>,
    #[arg(long, help = "Contaminant preset name (default: illumina_default)")]
    pub contaminant_preset: Option<String>,
    #[arg(
        long,
        help = "Enable contaminant k-mer removal when contaminant preset is set."
    )]
    pub enable_contaminant_removal: bool,
    #[arg(long)]
    pub no_qc_post: bool,
    #[arg(long)]
    pub force_merge: bool,
    #[arg(long, help = "Enable error correction stage (paired-end only)")]
    pub enable_correct: bool,
    #[arg(long, help = "Allow planned/out-of-scope stages in planning")]
    pub allow_planned: bool,
}
