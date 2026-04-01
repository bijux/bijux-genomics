use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub struct BenchPublicationTargetsArgs {
    pub kind: String,
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,
    #[arg(long, value_name = "CORPUS_ID")]
    pub corpus_id: String,
}

#[derive(Debug, Args)]
pub struct BenchCorpusFastqReportArgs {
    #[arg(long)]
    pub stage: String,
    #[arg(long, value_name = "CORPUS_ID")]
    pub corpus_id: String,
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,
    #[arg(
        long,
        value_name = "PATH",
        default_value = "docs/30-operations/benchmark"
    )]
    pub docs_root: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub run_root: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct BenchCorpusFastqPublicationStatusArgs {
    #[arg(long, value_name = "CORPUS_ID")]
    pub corpus_id: String,
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,
    #[arg(
        long,
        value_name = "PATH",
        default_value = "docs/30-operations/benchmark"
    )]
    pub docs_root: PathBuf,
}

#[derive(Debug, Args)]
pub struct BenchCorpusFastqPublishedDossiersArgs {
    #[arg(long, value_name = "CORPUS_ID")]
    pub corpus_id: String,
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,
    #[arg(
        long,
        value_name = "PATH",
        default_value = "docs/30-operations/benchmark"
    )]
    pub docs_root: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub run_root: Option<PathBuf>,
}
