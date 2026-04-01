use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub enum BenchConfigCommand {
    Validate(BenchConfigValidateArgs),
}

#[derive(Debug, Args)]
pub struct BenchConfigValidateArgs {
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub check_paths: bool,
}

#[derive(Debug, Args)]
pub struct BenchWorkspaceValueArgs {
    pub key: String,
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct BenchConfigJsonArgs {
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,
    #[arg(long, default_value = "full")]
    pub section: String,
}

#[derive(Debug, Args)]
pub struct BenchRepoChecksArgs {
    #[arg(long, value_name = "PATH", default_value = ".")]
    pub repo_root: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub json_out: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct BenchWriteScreenTaxonomyDatabaseLineageArgs {
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,
    #[arg(long, value_name = "CORPUS_ID")]
    pub corpus_id: String,
    #[arg(long, value_name = "PATH")]
    pub database_root: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    pub results_root: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    pub cache_root: Option<PathBuf>,
    #[arg(long, default_value = "taxonomy_reference")]
    pub database_catalog_id: String,
    #[arg(long, default_value = "taxonomy_db")]
    pub database_artifact_id: String,
    #[arg(long, default_value = "read_screening")]
    pub database_namespace: String,
    #[arg(long, default_value = "read_screening")]
    pub database_scope: String,
    #[arg(long, value_name = "PATH")]
    pub source_manifest: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    pub bootstrap_report: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    pub lineage_json: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct BenchNormalizeWorkspaceLayoutArgs {
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,
    #[arg(long, value_name = "CORPUS_ID")]
    pub corpus_id: String,
    #[arg(long, default_value_t = false)]
    pub confirm: bool,
}
