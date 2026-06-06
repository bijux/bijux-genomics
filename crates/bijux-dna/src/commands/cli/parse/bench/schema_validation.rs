use clap::Args;

pub use crate::commands::benchmark::vcf_schema_validation::BenchSchemaDomainArg;

#[derive(Debug, Args)]
pub struct BenchValidateSchemasArgs {
    #[arg(long, value_enum)]
    pub domain: BenchSchemaDomainArg,
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long)]
    pub shared_schema: Option<std::path::PathBuf>,
    #[arg(long)]
    pub stage_dir: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}
