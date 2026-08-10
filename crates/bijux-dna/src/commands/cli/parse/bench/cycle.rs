use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub enum BenchCycleCommand {
    /// Admit every independently ready benchmark output closure.
    Admit(BenchCycleAdmitArgs),
    /// Observe scheduler-active attempts once and collect accounting-ready transitions.
    Collect(BenchCycleCollectArgs),
    /// Plan every declared tool attempt for one permanent benchmark run.
    Plan(BenchCyclePlanArgs),
    /// Read the persisted lifecycle state for every attempt in one campaign.
    Status(BenchCycleStatusArgs),
}

#[derive(Debug, Args)]
pub struct BenchCycleAdmitArgs {
    #[arg(long, default_value = ".")]
    pub repository_root: PathBuf,
    #[arg(long)]
    pub study_root: PathBuf,
    #[arg(long)]
    pub benchmark_root: PathBuf,
    #[arg(long)]
    pub plan: PathBuf,
    #[arg(long)]
    pub admitted_at_utc: String,
}

#[derive(Debug, Args)]
pub struct BenchCycleCollectArgs {
    #[arg(long, default_value = ".")]
    pub repository_root: PathBuf,
    #[arg(long)]
    pub benchmark_root: PathBuf,
    #[arg(long)]
    pub plan: PathBuf,
    #[arg(long)]
    pub observed_at_utc: String,
}

#[derive(Debug, Args)]
pub struct BenchCycleStatusArgs {
    #[arg(long, default_value = ".")]
    pub repository_root: PathBuf,
    #[arg(long)]
    pub benchmark_root: PathBuf,
    #[arg(long)]
    pub plan: PathBuf,
    #[arg(long)]
    pub observed_at_utc: String,
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
