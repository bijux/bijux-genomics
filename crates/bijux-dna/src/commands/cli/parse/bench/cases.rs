use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub enum BenchCasesCommand {
    /// Plan and immutably publish the complete case matrix for one run.
    Plan(BenchCasesPlanArgs),
    /// Admit normalized observations against their exact case matrix.
    Admit(BenchCasesAdmitArgs),
    /// Aggregate an admitted case-observation closure.
    Aggregate(BenchCasesAggregateArgs),
}

#[derive(Debug, Args)]
pub struct BenchCasesPlanArgs {
    #[arg(long, default_value = ".")]
    pub repository_root: PathBuf,
    #[arg(long)]
    pub study_root: PathBuf,
    #[arg(long)]
    pub data_index: PathBuf,
    #[arg(long)]
    pub run_id: String,
    #[arg(long)]
    pub output: PathBuf,
}

#[derive(Debug, Args)]
pub struct BenchCasesAdmitArgs {
    #[arg(long)]
    pub matrix: PathBuf,
    #[arg(long)]
    pub observations: PathBuf,
    #[arg(long)]
    pub output: PathBuf,
}

#[derive(Debug, Args)]
pub struct BenchCasesAggregateArgs {
    #[arg(long)]
    pub matrix: PathBuf,
    #[arg(long)]
    pub observations: PathBuf,
    #[arg(long)]
    pub output: PathBuf,
}
