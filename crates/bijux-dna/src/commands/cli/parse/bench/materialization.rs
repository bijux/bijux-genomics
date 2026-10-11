use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct BenchAcquireFastqCorpusArgs {
    #[arg(long)]
    pub spec: PathBuf,
    #[arg(long)]
    pub output: PathBuf,
    #[arg(long)]
    pub retrieved_at_utc: String,
}

#[derive(Debug, Args)]
pub struct BenchAcquireSelectedFastqCollectionArgs {
    #[arg(long)]
    pub spec: PathBuf,
    #[arg(long)]
    pub collection_id: String,
    /// Immutable ENA record fixture used when the live metadata service is unavailable.
    #[arg(long)]
    pub offline_fixture: Option<PathBuf>,
    #[arg(long)]
    pub output: PathBuf,
    #[arg(long)]
    pub retrieved_at_utc: String,
}

#[derive(Debug, Subcommand)]
pub enum BenchCorpusMaterializationCommand {
    Plan(BenchCorpusMaterializationPlanArgs),
    Prepare(BenchCorpusMaterializationPrepareArgs),
    Submit(BenchCorpusMaterializationSubmitArgs),
    Status(BenchCorpusMaterializationStatusArgs),
    Collect(BenchCorpusMaterializationCollectArgs),
    CorrectEvidence(BenchCorpusMaterializationCorrectEvidenceArgs),
}

#[derive(Debug, Args)]
pub struct BenchCorpusMaterializationPlanArgs {
    #[arg(long)]
    pub repository: PathBuf,
    #[arg(long)]
    pub executable: PathBuf,
    #[arg(long)]
    pub plan: PathBuf,
    #[arg(long)]
    pub data_root: PathBuf,
    #[arg(long)]
    pub run_id: String,
    #[arg(long)]
    pub attempt_id: String,
    #[arg(long)]
    pub account: String,
    #[arg(long)]
    pub partition: String,
    #[arg(long, default_value_t = 4)]
    pub jobs: usize,
    #[arg(long)]
    pub memory_mb: u64,
    #[arg(long)]
    pub walltime: String,
    #[arg(long, default_value_t = 2)]
    pub retries: usize,
    #[arg(long, default_value_t = 120)]
    pub timeout_seconds: u64,
    #[arg(long)]
    pub admitted_at_utc: String,
}

#[derive(Debug, Args)]
pub struct BenchCorpusMaterializationCorrectEvidenceArgs {
    #[arg(long)]
    pub evidence: PathBuf,
    #[arg(long)]
    pub output: PathBuf,
    #[arg(long)]
    pub observed_local_time: String,
    #[arg(long)]
    pub source_timezone: String,
    #[arg(long)]
    pub corrected_at_utc: String,
    #[arg(long)]
    pub source_commit: String,
    #[arg(long)]
    pub reason: String,
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
    #[arg(long)]
    pub repository_commit: String,
    #[arg(long, default_value_t = 4)]
    pub jobs: usize,
    #[arg(long, default_value_t = 2)]
    pub retries: usize,
    #[arg(long, default_value_t = 120)]
    pub timeout_seconds: u64,
    #[arg(long)]
    pub admitted_at_utc: String,
}
