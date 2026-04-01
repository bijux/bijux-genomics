use std::path::PathBuf;

use clap::Args;

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
    #[arg(long)]
    pub min_length: Option<u32>,
    #[arg(long)]
    pub quality_cutoff: Option<u32>,
    #[arg(long)]
    pub n_policy: Option<String>,
    #[arg(long)]
    pub adapter_policy: Option<String>,
    #[arg(long)]
    pub polyx_policy: Option<String>,
    #[arg(long)]
    pub contaminant_policy: Option<String>,
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
    pub trim_polyg: Option<bool>,
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
    pub damage_mode: Option<String>,
    #[arg(
        long,
        help = "Execution policy: policy_derived | explicit_terminal_trim | preserve_udg_trimmed_ends"
    )]
    pub execution_policy: Option<String>,
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
    pub strict: bool,
    #[arg(long)]
    pub threads: Option<u32>,
    #[arg(long)]
    pub validation_mode: Option<String>,
    #[arg(long)]
    pub pair_sync_policy: Option<String>,
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
}
