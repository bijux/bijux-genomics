use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub enum BenchCycleCommand {
    /// Admit every independently ready benchmark output closure.
    Admit(BenchCycleAdmitArgs),
    /// Collect, admit, prepare, and submit the bounded ready frontier once.
    Advance(BenchCycleAdvanceArgs),
    /// Publish a reviewer-facing book from current admitted publication receipts.
    Book(BenchCycleBookArgs),
    /// Observe scheduler-active attempts once and collect accounting-ready transitions.
    Collect(BenchCycleCollectArgs),
    /// Plan every declared tool attempt for one permanent benchmark run.
    Plan(BenchCyclePlanArgs),
    /// Publish one source-locked campaign report from admitted attempt evidence.
    Publish(BenchCyclePublishArgs),
    /// Compare regenerated semantic artifacts with a selected publication.
    Replay(BenchCycleReplayArgs),
    /// Read the persisted lifecycle state for every attempt in one campaign.
    Status(BenchCycleStatusArgs),
}

#[derive(Debug, Args)]
pub struct BenchCycleAdvanceArgs {
    #[arg(long, default_value = ".")]
    pub repository_root: PathBuf,
    #[arg(long)]
    pub study_root: PathBuf,
    #[arg(long)]
    pub data_index: PathBuf,
    #[arg(long)]
    pub inventory: PathBuf,
    #[arg(long)]
    pub benchmark_root: PathBuf,
    #[arg(long)]
    pub plan: PathBuf,
    #[arg(long)]
    pub operator: String,
    #[arg(long)]
    pub advanced_at_utc: String,
    #[arg(long)]
    pub account: String,
    #[arg(long)]
    pub partition: String,
    #[arg(long, default_value_t = 32)]
    pub maximum_submissions: usize,
}

#[derive(Debug, Args)]
pub struct BenchCycleReplayArgs {
    #[arg(long, default_value = ".")]
    pub repository_root: PathBuf,
    #[arg(long)]
    pub study_root: PathBuf,
    #[arg(long)]
    pub run_id: String,
    #[arg(long)]
    pub replay_id: String,
    #[arg(long)]
    pub replayed_at_utc: String,
    #[arg(long)]
    pub run_definition_uri: String,
    #[arg(long, required = true)]
    pub case_definition_uri: Vec<String>,
    #[arg(long, required = true)]
    pub immutable_input_uri: Vec<String>,
    #[arg(long)]
    pub observed_decision_uri: Option<String>,
    #[arg(long)]
    pub observed_report_uri: Option<String>,
    #[arg(long)]
    pub observed_evidence_book_uri: Option<String>,
    #[arg(long)]
    pub difference: Vec<String>,
    #[arg(long, required = true)]
    pub limitation: Vec<String>,
    #[arg(long)]
    pub output: PathBuf,
}

#[derive(Debug, Args)]
pub struct BenchCycleBookArgs {
    #[arg(long, default_value = ".")]
    pub repository_root: PathBuf,
    #[arg(long)]
    pub study_root: PathBuf,
    #[arg(long)]
    pub book_id: String,
    #[arg(long)]
    pub generated_at_utc: String,
    #[arg(long, required = true)]
    pub limitation: Vec<String>,
    #[arg(long)]
    pub output: PathBuf,
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

#[derive(Debug, Args)]
pub struct BenchCyclePublishArgs {
    #[arg(long, default_value = ".")]
    pub repository_root: PathBuf,
    #[arg(long, default_value = ".")]
    pub presentation_repository_root: PathBuf,
    #[arg(long)]
    pub study_root: PathBuf,
    #[arg(long)]
    pub benchmark_root: PathBuf,
    #[arg(long)]
    pub plan: PathBuf,
    #[arg(long)]
    pub output_root: PathBuf,
}
