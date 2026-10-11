use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub enum BenchCatalogRegistryCommand {
    Publish(BenchCatalogRegistryPublishArgs),
}

#[derive(Debug, Args)]
pub struct BenchCatalogRegistryPublishArgs {
    #[arg(long)]
    pub data_root: PathBuf,
    #[arg(long)]
    pub source_commit: String,
}

#[derive(Debug, Args)]
pub struct BenchDataIndexArgs {
    #[arg(long)]
    pub catalog_registry: PathBuf,
    #[arg(long)]
    pub data_root: PathBuf,
    #[arg(long)]
    pub output: PathBuf,
    #[arg(long)]
    pub report: PathBuf,
    #[arg(long)]
    pub created_at_utc: String,
}
