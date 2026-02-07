#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ObjectiveArg {
    Speed,
    Memory,
    Retention,
    Balanced,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ScientificPresetArg {
    #[value(name = "ancient_dna")]
    AncientDna,
    #[value(name = "amplicon")]
    Amplicon,
    #[value(name = "metagenomic")]
    Metagenomic,
    #[value(name = "wgs_standard")]
    WgsStandard,
}

impl ObjectiveArg {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            ObjectiveArg::Speed => "speed",
            ObjectiveArg::Memory => "memory",
            ObjectiveArg::Retention => "retention",
            ObjectiveArg::Balanced => "balanced",
        }
    }
}

impl From<ObjectiveArg> for bijux_api::v1::api::bench::Objective {
    fn from(value: ObjectiveArg) -> Self {
        match value {
            ObjectiveArg::Speed => bijux_api::v1::api::bench::Objective::Speed,
            ObjectiveArg::Memory => bijux_api::v1::api::bench::Objective::Memory,
            ObjectiveArg::Retention => bijux_api::v1::api::bench::Objective::Retention,
            ObjectiveArg::Balanced => bijux_api::v1::api::bench::Objective::Balanced,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum BenchCorpusArg {
    #[value(name = "fastq_5set")]
    Fastq5Set,
}

impl From<BenchCorpusArg> for bijux_api::v1::api::bench::BenchCorpusId {
    fn from(value: BenchCorpusArg) -> Self {
        match value {
            BenchCorpusArg::Fastq5Set => bijux_api::v1::api::bench::BenchCorpusId::Fastq5Set,
        }
    }
}

#[derive(Debug, Args, Clone, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct CommonArgs {
    #[arg(long)]
    pub list_tools: bool,
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long, help = "Allow silver-tier tools")]
    pub allow_silver: bool,
    #[arg(long, help = "Allow experimental-tier tools")]
    pub allow_experimental: bool,
}

#[derive(Debug, Args, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct FastqPreprocessArgs {
    #[command(flatten)]
    pub common: CommonArgs,
    #[arg(long, help = "Pipeline profile id (default, minimal)")]
    pub pipeline_profile: Option<String>,
    #[arg(long)]
    pub list_adapter_presets: bool,
    #[arg(long)]
    pub list_adapters: bool,
    #[arg(long)]
    pub env: Option<String>,
    #[arg(long, alias = "sample")]
    pub sample_id: Option<String>,
    #[arg(long)]
    pub r1: Option<PathBuf>,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(long)]
    pub out: Option<PathBuf>,
    #[arg(long)]
    pub strict: bool,
    #[arg(long)]
    pub auto: bool,
    #[arg(long, value_enum, default_value_t = ObjectiveArg::Balanced)]
    pub objective: ObjectiveArg,
    #[arg(
        long,
        value_enum,
        help = "Scientific preset profile (ancient_dna, amplicon, metagenomic, wgs_standard)"
    )]
    pub scientific_preset: Option<ScientificPresetArg>,
    #[arg(long, value_enum)]
    pub bench_corpus: Option<BenchCorpusArg>,
    #[arg(long)]
    pub allow_partial: bool,
    #[arg(long, default_value_t = 1)]
    pub jobs: u32,
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
    #[arg(long, value_name = "PATH", help = "Alignment boundary BAM path (cross-domain profiles)")]
    pub alignment_bam: Option<PathBuf>,
    #[arg(long, value_name = "PATH", help = "Alignment boundary BAI path (optional)")]
    pub alignment_bai: Option<PathBuf>,
    #[arg(long, value_name = "PATH", help = "Alignment boundary reference (optional)")]
    pub alignment_reference: Option<PathBuf>,
    #[arg(long, help = "Alignment boundary read-group policy (optional)")]
    pub alignment_rg_policy: Option<String>,
    #[arg(
        long,
        value_name = "KEY=VALUE",
        help = "Alignment boundary aligner metadata (repeatable)"
    )]
    pub alignment_meta: Vec<String>,
}

#[derive(Debug, Args, Clone)]
pub struct FastqBenchmarkArgs {
    #[arg(long, default_value = "runs")]
    pub runs: PathBuf,
    #[arg(long)]
    pub stage: String,
    #[arg(long, value_enum, default_value_t = ObjectiveArg::Balanced)]
    pub objective: ObjectiveArg,
}

#[derive(Debug, Args, Clone)]
pub struct FastqRunArgs {
    #[command(flatten)]
    pub args: FastqPreprocessArgs,
}

#[derive(Debug, Args, Clone)]
pub struct FastqCompareArgs {
    #[arg(long)]
    pub run_a: String,
    #[arg(long)]
    pub run_b: String,
    #[arg(long, default_value = "runs")]
    pub search_root: PathBuf,
    #[arg(long)]
    pub baseline: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum FastqCommand {
    #[command(about = "List FASTQ stages.")]
    ListStages,
    #[command(about = "List FASTQ stage ids and versions.")]
    Stages,
    #[command(about = "Check FASTQ prerequisites (runner, cache, image catalog).")]
    Doctor,
    #[command(about = "List tools for a FASTQ stage.")]
    ListTools {
        #[arg(long)]
        stage: String,
    },
    #[command(about = "Explain a FASTQ stage or pipeline.")]
    Explain {
        stage: String,
    },
    #[command(
        about = "Filter FASTQ reads.",
        after_help = "Examples:\n  bijux fastq filter --r1 reads.fastq.gz --out artifacts --sample-id SAMPLE --tools fastp\n  bijux fastq filter --list-tools"
    )]
    Filter(FastqFilterArgs),
    #[command(
        about = "Merge paired-end FASTQ reads.",
        after_help = "Example:\n  bijux fastq merge --r1 reads_1.fastq.gz --r2 reads_2.fastq.gz --out artifacts --sample-id SAMPLE --tools vsearch\n\nNext stages: filter -> stats"
    )]
    Merge(CommonArgs),
    #[command(
        about = "Trim FASTQ reads (quality/adapters) and emit canonical outputs.",
        after_help = "Example:\n  bijux fastq trim --r1 reads.fastq.gz --out artifacts --sample-id SAMPLE --tools fastp\n\nNext stages: filter -> stats"
    )]
    Trim(FastqTrimArgs),
    Contam(CommonArgs),
    #[command(
        about = "Run the FASTQ preprocess pipeline (validate → trim → filter → stats).",
        after_help = "Examples:\n  bijux fastq preprocess --r1 reads.fastq.gz --out artifacts --sample-id SAMPLE\n  bijux fastq preprocess --auto --objective speed --bench-corpus fastq_5set --r1 reads.fastq.gz --out artifacts --sample-id SAMPLE\n  bijux fastq preprocess --list-tools"
    )]
    Preprocess(FastqPreprocessArgs),
    #[command(
        about = "Run the FASTQ pipeline (validate → trim → filter → stats).",
        after_help = "Examples:\n  bijux fastq run --r1 reads.fastq.gz --out artifacts --sample-id SAMPLE\n  bijux fastq run --auto --objective speed --bench-corpus fastq_5set --r1 reads.fastq.gz --out artifacts --sample-id SAMPLE"
    )]
    Run(FastqRunArgs),
    #[command(
        name = "stats-neutral",
        alias = "stats",
        about = "Summarize FASTQ read statistics (neutral).",
        after_help = "Example:\n  bijux fastq stats-neutral --r1 reads.fastq.gz --out artifacts --sample-id SAMPLE --tools seqkit_stats\n\nNext stages: report/compare"
    )]
    StatsNeutral(CommonArgs),
    Umi(CommonArgs),
    #[command(name = "error-correct")]
    ErrorCorrect(CommonArgs),
    Qc(CommonArgs),
    #[command(
        name = "validate-pre",
        alias = "validate",
        about = "Validate FASTQ reads (pre).",
        after_help = "Examples:\n  bijux fastq validate-pre --r1 reads.fastq.gz --out artifacts --sample-id SAMPLE --tools fastqvalidator_official\n  bijux fastq validate-pre --list-tools"
    )]
    ValidatePre(FastqValidateArgs),
    #[command(about = "Benchmark existing FASTQ runs without re-execution.")]
    Benchmark(FastqBenchmarkArgs),
    #[command(about = "Analyze FASTQ runs without re-execution.")]
    Analyze(FastqBenchmarkArgs),
    #[command(about = "Compare two FASTQ runs.")]
    Compare(FastqCompareArgs),
    Align(CommonArgs),
}

#[derive(Debug, Args, Clone)]
pub struct FastqTrimArgs {
    #[command(flatten)]
    pub common: CommonArgs,
    #[arg(long)]
    pub list_adapter_presets: bool,
    #[arg(long)]
    pub list_adapters: bool,
    #[arg(long)]
    pub env: Option<String>,
    #[arg(long, alias = "sample")]
    pub sample_id: Option<String>,
    #[arg(long)]
    pub r1: Option<PathBuf>,
    #[arg(long)]
    pub out: Option<PathBuf>,
    #[arg(long, value_delimiter = ',')]
    pub tools: Vec<String>,
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

#[derive(Debug, Args, Clone)]
pub struct FastqFilterArgs {
    #[command(flatten)]
    pub common: CommonArgs,
    #[arg(long)]
    pub env: Option<String>,
    #[arg(long, alias = "sample")]
    pub sample_id: Option<String>,
    #[arg(long)]
    pub r1: Option<PathBuf>,
    #[arg(long)]
    pub out: Option<PathBuf>,
    #[arg(long, value_delimiter = ',')]
    pub tools: Vec<String>,
    #[arg(long)]
    pub max_n: Option<u32>,
    #[arg(long = "low-complexity-threshold")]
    pub low_complexity_threshold: Option<f64>,
    #[arg(long)]
    pub kmer_ref: Option<PathBuf>,
}

#[derive(Debug, Args, Clone)]
pub struct FastqValidateArgs {
    #[command(flatten)]
    pub common: CommonArgs,
    #[arg(long)]
    pub env: Option<String>,
    #[arg(long, alias = "sample")]
    pub sample_id: Option<String>,
    #[arg(long)]
    pub r1: Option<PathBuf>,
    #[arg(long)]
    pub out: Option<PathBuf>,
    #[arg(long, value_delimiter = ',')]
    pub tools: Vec<String>,
    #[arg(long)]
    pub strict: bool,
}
