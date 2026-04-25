use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "bijux-dna-science")]
pub struct ScienceCli {
    #[arg(long, default_value = ".")]
    pub workspace_root: PathBuf,

    #[command(subcommand)]
    pub command: ScienceCommand,
}

#[derive(Debug, Subcommand)]
pub enum ScienceCommand {
    Validate,
    Build,
    Trace {
        #[arg(long)]
        stage: Option<String>,
        #[arg(long)]
        tool: Option<String>,
    },
    Release {
        #[arg(long)]
        release_id: String,
    },
}
