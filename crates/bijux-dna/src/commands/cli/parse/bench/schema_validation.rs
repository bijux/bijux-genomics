use clap::{Args, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum BenchSchemaDomainArg {
    Fastq,
    Bam,
    Vcf,
}

#[derive(Debug, Args)]
pub struct BenchValidateSchemasArgs {
    #[arg(long, value_enum, value_delimiter = ',', num_args = 1..)]
    pub domain: Vec<BenchSchemaDomainArg>,
    #[arg(long)]
    pub schema_root: Option<std::path::PathBuf>,
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long)]
    pub shared_schema: Option<std::path::PathBuf>,
    #[arg(long)]
    pub stage_dir: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}
