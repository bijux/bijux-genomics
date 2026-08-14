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
    /// Issue one immutable publication receipt from a passing campaign closure.
    Receipt(BenchCycleReceiptArgs),
    /// Compare regenerated semantic artifacts with a selected publication.
    Replay(BenchCycleReplayArgs),
    /// Retry only non-passing terminal tips from case-scoped attempt lineage.
    Retry(BenchCycleRetryArgs),
    /// Read the persisted lifecycle state for every attempt in one campaign.
    Status(BenchCycleStatusArgs),
    /// Terminalize non-active attempts from an ancestor case campaign.
    Supersede(BenchCycleSupersedeArgs),
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
    pub case_matrix: Option<PathBuf>,
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
    /// Candidate publication receipt used before final tracked selection.
    #[arg(long, requires = "evidence_book_uri")]
    pub publication_receipt_uri: Option<String>,
    /// Candidate evidence book that admits the explicit publication receipt.
    #[arg(long, requires = "publication_receipt_uri")]
    pub evidence_book_uri: Option<String>,
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
    /// Immutable candidate publication receipts to include before final pointer selection.
    #[arg(long)]
    pub publication_receipt_uri: Vec<String>,
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
    pub case_matrix: Option<PathBuf>,
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
    pub case_matrix: Option<PathBuf>,
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
    pub case_matrix: Option<PathBuf>,
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
    pub case_matrix: Option<PathBuf>,
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
pub struct BenchCycleRetryArgs {
    #[arg(long, default_value = ".")]
    pub repository_root: PathBuf,
    #[arg(long)]
    pub benchmark_root: PathBuf,
    #[arg(long)]
    pub case_matrix: PathBuf,
    #[arg(long)]
    pub prior_plan: PathBuf,
    #[arg(long)]
    pub prior_case_matrix: PathBuf,
    #[arg(long)]
    pub attempt_id: String,
    #[arg(long)]
    pub observed_at_utc: String,
    #[arg(long)]
    pub output: PathBuf,
}

#[derive(Debug, Args)]
pub struct BenchCycleSupersedeArgs {
    #[arg(long, default_value = ".")]
    pub repository_root: PathBuf,
    #[arg(long)]
    pub benchmark_root: PathBuf,
    #[arg(long)]
    pub prior_plan: PathBuf,
    #[arg(long)]
    pub prior_case_matrix: PathBuf,
    #[arg(long)]
    pub superseded_at_utc: String,
    #[arg(long)]
    pub reason: String,
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
    pub case_matrix: Option<PathBuf>,
    #[arg(long)]
    pub output_root: PathBuf,
}

#[derive(Debug, Args)]
pub struct BenchCycleReceiptArgs {
    #[arg(long, default_value = ".")]
    pub repository_root: PathBuf,
    #[arg(long)]
    pub study_root: PathBuf,
    #[arg(long)]
    pub benchmark_root: PathBuf,
    #[arg(long)]
    pub plan_uri: String,
    #[arg(long)]
    pub case_matrix_uri: String,
    #[arg(long)]
    pub report_uri: String,
    #[arg(long)]
    pub decision_uri: String,
    #[arg(long)]
    pub publication_id: String,
    #[arg(long)]
    pub published_at_utc: String,
    #[arg(long, required = true)]
    pub limitation: Vec<String>,
    #[arg(long)]
    pub output: PathBuf,
}
