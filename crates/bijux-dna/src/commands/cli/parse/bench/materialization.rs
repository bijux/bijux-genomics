use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub struct BenchMaterializeFastqCorpusArgs {
    #[arg(long)]
    pub plan: PathBuf,
    #[arg(long)]
    pub data_root: PathBuf,
    #[arg(long, default_value_t = 4)]
    pub jobs: usize,
    #[arg(long, default_value_t = 2)]
    pub retries: usize,
    #[arg(long, default_value_t = 120)]
    pub timeout_seconds: u64,
    #[arg(long)]
    pub admitted_at_utc: String,
}
