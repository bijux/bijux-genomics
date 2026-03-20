
#[derive(Debug, Subcommand)]
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
    #[command(name = "trim-reads", visible_alias = "trim")]
    Trim(BenchFastqTrimArgs),
    #[command(name = "trim-polyg-tails")]
    TrimPolygTails(BenchFastqTrimPolygArgs),
    #[command(name = "trim-terminal-damage")]
    TrimTerminalDamage(BenchFastqTrimTerminalDamageArgs),
    #[command(name = "validate-reads", visible_alias = "validate")]
    Validate(BenchFastqValidateArgs),
    #[command(name = "detect-adapters")]
    DetectAdapters(BenchFastqDetectAdaptersArgs),
    #[command(name = "profile-read-lengths")]
    ProfileReadLengths(BenchFastqProfileReadLengthsArgs),
    Filter(BenchFastqFilterArgs),
    #[command(name = "filter-low-complexity")]
    FilterLowComplexity(BenchFastqFilterLowComplexityArgs),
    Merge(BenchFastqMergeArgs),
    #[command(name = "remove-duplicates")]
    RemoveDuplicates(BenchFastqRemoveDuplicatesArgs),
    #[command(name = "remove-chimeras")]
    RemoveChimeras(BenchFastqRemoveChimerasArgs),
    #[command(name = "normalize-primers")]
    NormalizePrimers(BenchFastqNormalizePrimersArgs),
    #[command(name = "infer-asvs")]
    InferAsvs(BenchFastqInferAsvsArgs),
    #[command(name = "cluster-otus")]
    ClusterOtus(BenchFastqClusterOtusArgs),
    #[command(name = "normalize-abundance")]
    NormalizeAbundance(BenchFastqNormalizeAbundanceArgs),
    Correct(BenchFastqCorrectArgs),
    #[command(name = "report-qc", visible_alias = "qc-post", alias = "qc2")]
    ReportQc(BenchFastqQcPostArgs),
    Umi(BenchFastqUmiArgs),
    #[command(name = "index-reference")]
    IndexReference(BenchFastqIndexReferenceArgs),
    #[command(name = "screen-taxonomy", visible_alias = "screen")]
    Screen(BenchFastqScreenArgs),
    #[command(name = "deplete-host")]
    DepleteHost(BenchFastqDepleteHostArgs),
    #[command(name = "deplete-reference-contaminants")]
    DepleteReferenceContaminants(BenchFastqDepleteReferenceContaminantsArgs),
    #[command(name = "deplete-rrna")]
    DepleteRrna(BenchFastqDepleteRrnaArgs),
    #[command(name = "profile-reads", visible_alias = "stats")]
    Stats(BenchFastqStatsArgs),
    #[command(name = "profile-overrepresented-sequences", visible_alias = "overrepresented")]
    ProfileOverrepresentedSequences(BenchFastqProfileOverrepresentedArgs),
    Preprocess(BenchFastqPreprocessArgs),
}

#[derive(Debug, Args)]
pub struct BenchFastqTrimArgs {
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
pub struct BenchFastqTrimPolygArgs {
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
    #[arg(long, help = "PolyX preset name (default: illumina_twocolor)")]
    pub polyx_preset: Option<String>,
    #[arg(long)]
    pub min_polyg_run: Option<u32>,
}

#[derive(Debug, Args)]
pub struct BenchFastqTrimTerminalDamageArgs {
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
    #[arg(long)]
    pub damage_mode: Option<String>,
    #[arg(long)]
    pub trim_5p_bases: Option<u32>,
    #[arg(long)]
    pub trim_3p_bases: Option<u32>,
}

#[derive(Debug, Args)]
pub struct BenchFastqValidateArgs {
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
    #[arg(long)]
    pub strict: bool,
}

#[derive(Debug, Args)]
pub struct BenchFastqDetectAdaptersArgs {
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
pub struct BenchFastqProfileReadLengthsArgs {
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
pub struct BenchFastqFilterArgs {
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
    #[arg(long)]
    pub max_n: Option<u32>,
    #[arg(long = "low-complexity-threshold")]
    pub low_complexity_threshold: Option<f64>,
    #[arg(long)]
    pub kmer_ref: Option<PathBuf>,
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
    pub entropy_threshold: Option<f64>,
    #[arg(long)]
    pub polyx_threshold: Option<u32>,
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
pub struct BenchFastqRemoveDuplicatesArgs {
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
pub struct BenchFastqRemoveChimerasArgs {
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
pub struct BenchFastqNormalizePrimersArgs {
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
pub struct BenchFastqInferAsvsArgs {
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
pub struct BenchFastqNormalizeAbundanceArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub table: PathBuf,
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
pub struct BenchFastqQcPostArgs {
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
pub struct BenchFastqUmiArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, help = "UMI barcode pattern passed to umi_tools extract")]
    pub umi_pattern: String,
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
pub struct BenchFastqClusterOtusArgs {
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
pub struct BenchFastqIndexReferenceArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub reference_fasta: PathBuf,
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
pub struct BenchFastqDepleteHostArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(long)]
    pub reference_index: PathBuf,
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
pub struct BenchFastqDepleteReferenceContaminantsArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(long)]
    pub reference_index: PathBuf,
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
pub struct BenchFastqDepleteRrnaArgs {
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
pub struct BenchFastqStatsArgs {
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
pub struct BenchFastqProfileOverrepresentedArgs {
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
pub struct BenchFastqPreprocessArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long, help = "Pipeline profile id (default, minimal)")]
    pub pipeline_profile: Option<String>,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(long, value_name = "PATH", help = "Reference FASTA for reference-guided FASTQ stages")]
    pub reference_fasta: Option<PathBuf>,
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
