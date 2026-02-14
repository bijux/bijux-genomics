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
    #[arg(long, default_value = "sample")]
    pub sample_name: String,
    #[arg(long)]
    pub reference_fasta: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub production_profile: bool,
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long, default_value_t = 5_000_000)]
    pub chunk_window_size_bp: u64,
    #[arg(long, default_value_t = 100_000)]
    pub chunk_overlap_bp: u64,
    #[arg(long, value_delimiter = ',')]
    pub chunk_chr_include: Vec<String>,
    #[arg(long, value_delimiter = ',')]
    pub chunk_chr_exclude: Vec<String>,
    #[arg(long, default_value_t = 8)]
    pub max_parallel_chunks: usize,
    #[arg(long, default_value_t = false)]
    pub partial_allowed: bool,
    #[arg(long)]
    pub rerun_chunk: Option<String>,
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
