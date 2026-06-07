use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub enum BenchPathsCommand {
    Validate(BenchPathsValidateArgs),
}

#[derive(Debug, Args)]
pub struct BenchPathsValidateArgs {
    #[arg(long, default_value_t = false)]
    pub strict: bool,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}
