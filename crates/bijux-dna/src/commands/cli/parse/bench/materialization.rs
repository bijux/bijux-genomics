use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub enum BenchCorpusMaterializationCommand {
    Prepare(BenchCorpusMaterializationPrepareArgs),
    Submit(BenchCorpusMaterializationSubmitArgs),
    Status(BenchCorpusMaterializationStatusArgs),
    Collect(BenchCorpusMaterializationCollectArgs),
}

#[derive(Debug, Args)]
pub struct BenchCorpusMaterializationPrepareArgs {
    #[arg(long)]
    pub specification: PathBuf,
    #[arg(long)]
    pub submitted_at_utc: String,
}

#[derive(Debug, Args)]
pub struct BenchCorpusMaterializationSubmitArgs {
    #[arg(long)]
    pub specification: PathBuf,
    #[command(flatten)]
    pub slurm: BenchCorpusMaterializationSlurmArgs,
}

#[derive(Debug, Args)]
pub struct BenchCorpusMaterializationStatusArgs {
    #[arg(long)]
    pub specification: PathBuf,
    #[arg(long)]
    pub observed_at_utc: String,
    #[command(flatten)]
    pub slurm: BenchCorpusMaterializationSlurmArgs,
}

#[derive(Debug, Args)]
pub struct BenchCorpusMaterializationCollectArgs {
    #[arg(long)]
    pub specification: PathBuf,
    #[arg(long)]
    pub collected_at_utc: String,
    #[command(flatten)]
    pub slurm: BenchCorpusMaterializationSlurmArgs,
}

#[derive(Debug, Args)]
pub struct BenchCorpusMaterializationSlurmArgs {
    #[arg(long, default_value = "sbatch")]
    pub sbatch: PathBuf,
    #[arg(long, default_value = "squeue")]
    pub squeue: PathBuf,
    #[arg(long, default_value = "sacct")]
    pub sacct: PathBuf,
    #[arg(long, default_value = "scancel")]
    pub scancel: PathBuf,
}

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
