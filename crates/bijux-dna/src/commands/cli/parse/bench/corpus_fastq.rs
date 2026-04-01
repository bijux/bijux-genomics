use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub struct BenchCorpusFastqArgs {
    #[arg(long)]
    pub stage: String,
    #[arg(long, value_name = "CORPUS_ID")]
    pub corpus_id: String,
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    pub publication_config: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    pub corpus_root: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    pub out_root: Option<PathBuf>,
    #[arg(long, value_delimiter = ',')]
    pub tools: Vec<String>,
    #[arg(long, default_value_t = 1)]
    pub threads: u32,
    #[arg(long, default_value_t = 1)]
    pub jobs: u32,
    #[arg(long, default_value_t = 1)]
    pub sample_jobs: usize,
    #[arg(long, default_value_t = 0)]
    pub sample_limit: usize,
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    pub resume: bool,
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
    #[arg(long = "stage-arg")]
    pub stage_args: Vec<String>,
    #[arg(long = "manifest-arg")]
    pub manifest_args: Vec<String>,
}
