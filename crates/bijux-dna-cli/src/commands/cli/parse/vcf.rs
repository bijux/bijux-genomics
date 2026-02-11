#[derive(Debug, Args, Clone)]
pub struct VcfRunArgs {
    #[arg(long, default_value = "vcf-to-vcf__minimal__v1")]
    pub profile: String,
    #[arg(long)]
    pub vcf: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long)]
    pub tool: Option<String>,
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Subcommand)]
pub enum VcfCommand {
    Plan {
        #[arg(long, default_value = "vcf-to-vcf__minimal__v1")]
        profile: String,
    },
    Explain {
        #[arg(long, default_value = "vcf-to-vcf__minimal__v1")]
        profile: String,
    },
    Run(VcfRunArgs),
}
