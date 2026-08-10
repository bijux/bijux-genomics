use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub enum BenchCycleCommand {
    /// Plan every declared tool attempt for one permanent benchmark run.
    Plan(BenchCyclePlanArgs),
}

#[derive(Debug, Args)]
pub struct BenchCyclePlanArgs {
    #[arg(long, default_value = ".")]
    pub repository_root: PathBuf,
    #[arg(long)]
    pub run_id: String,
    #[arg(long)]
    pub campaign_id: String,
    #[arg(long)]
    pub attempt_id: String,
    #[arg(long)]
    pub observed_at_utc: String,
    #[arg(long)]
    pub output: PathBuf,
}
