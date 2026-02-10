// imports provided by core parser module

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum BamStageArg {
    Align,
    Validate,
    QcPre,
    Filter,
    Markdup,
    Complexity,
    Coverage,
    Damage,
    Authenticity,
    Contamination,
    Sex,
    BiasMitigation,
    Recalibration,
    Haplogroups,
    Genotyping,
    Kinship,
}

impl BamStageArg {
    #[must_use]
    pub fn stage(self) -> bijux_dna_api::v1::api::bench::BamStage {
        match self {
            BamStageArg::Align => bijux_dna_api::v1::api::bench::BamStage::Align,
            BamStageArg::Validate => bijux_dna_api::v1::api::bench::BamStage::Validate,
            BamStageArg::QcPre => bijux_dna_api::v1::api::bench::BamStage::QcPre,
            BamStageArg::Filter => bijux_dna_api::v1::api::bench::BamStage::Filter,
            BamStageArg::Markdup => bijux_dna_api::v1::api::bench::BamStage::Markdup,
            BamStageArg::Complexity => bijux_dna_api::v1::api::bench::BamStage::Complexity,
            BamStageArg::Coverage => bijux_dna_api::v1::api::bench::BamStage::Coverage,
            BamStageArg::Damage => bijux_dna_api::v1::api::bench::BamStage::Damage,
            BamStageArg::Authenticity => bijux_dna_api::v1::api::bench::BamStage::Authenticity,
            BamStageArg::Contamination => bijux_dna_api::v1::api::bench::BamStage::Contamination,
            BamStageArg::Sex => bijux_dna_api::v1::api::bench::BamStage::Sex,
            BamStageArg::BiasMitigation => bijux_dna_api::v1::api::bench::BamStage::BiasMitigation,
            BamStageArg::Recalibration => bijux_dna_api::v1::api::bench::BamStage::Recalibration,
            BamStageArg::Haplogroups => bijux_dna_api::v1::api::bench::BamStage::Haplogroups,
            BamStageArg::Genotyping => bijux_dna_api::v1::api::bench::BamStage::Genotyping,
            BamStageArg::Kinship => bijux_dna_api::v1::api::bench::BamStage::Kinship,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum UdgModelArg {
    NonUdg,
    HalfUdg,
    Udg,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ReadGroupPolicyArg {
    Preserve,
    Merge,
    Regenerate,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ContaminationScopeArg {
    Mito,
    Nuclear,
    Both,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum BqsrModeArg {
    Standard,
    Skip,
    EmitOnly,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ExpectedSexArg {
    Xx,
    Xy,
    Unknown,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OpticalDuplicatePolicyArg {
    None,
    MarkOnly,
    Remove,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum UmiPolicyArg {
    Ignore,
    UseTag,
    Collapse,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum DuplicateActionArg {
    Mark,
    Remove,
}

#[derive(Debug, Args, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct BamRunArgs {
    #[arg(long, value_enum, default_value_t = BamStageArg::Validate)]
    pub stage: BamStageArg,
    #[arg(long, default_value = "bam-to-bam__default__v1")]
    pub profile: String,
    #[arg(long, alias = "sample")]
    pub sample_id: Option<String>,
    #[arg(long)]
    pub r1: Option<PathBuf>,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(long)]
    pub bam: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long)]
    pub tool: Option<String>,
    #[arg(long)]
    pub bai: Option<PathBuf>,
    #[arg(long)]
    pub reference: Option<PathBuf>,
    #[arg(long)]
    pub regions: Option<PathBuf>,
    #[arg(long, value_enum)]
    pub udg_model: Option<UdgModelArg>,
    #[arg(long)]
    pub pmd_threshold_5p: Option<f64>,
    #[arg(long)]
    pub pmd_threshold_3p: Option<f64>,
    #[arg(long)]
    pub trim_5p: Option<u8>,
    #[arg(long)]
    pub trim_3p: Option<u8>,
    #[arg(long, value_enum)]
    pub contamination_scope: Option<ContaminationScopeArg>,
    #[arg(long)]
    pub contamination_panel: Vec<String>,
    #[arg(long)]
    pub contamination_prior: Option<f64>,
    #[arg(long)]
    pub sex_specific_contamination: bool,
    #[arg(long)]
    pub contamination_assumptions: Option<String>,
    #[arg(long, value_enum)]
    pub expected_sex: Option<ExpectedSexArg>,
    #[arg(long, default_value = "rxy")]
    pub sex_method: String,
    #[arg(long)]
    pub min_mapq: Option<u8>,
    #[arg(long)]
    pub min_length: Option<u32>,
    #[arg(long, value_delimiter = ',')]
    pub include_flags: Vec<u16>,
    #[arg(long, value_delimiter = ',')]
    pub exclude_flags: Vec<u16>,
    #[arg(long)]
    pub remove_duplicates: bool,
    #[arg(long)]
    pub base_quality_threshold: Option<u8>,
    #[arg(long, value_enum)]
    pub optical_duplicates: Option<OpticalDuplicatePolicyArg>,
    #[arg(long, value_enum)]
    pub umi_policy: Option<UmiPolicyArg>,
    #[arg(long, value_enum)]
    pub duplicate_action: Option<DuplicateActionArg>,
    #[arg(long)]
    pub complexity_min_reads: Option<u64>,
    #[arg(long, value_delimiter = ',')]
    pub complexity_projection_points: Vec<u64>,
    #[arg(long, value_delimiter = ',')]
    pub depth_thresholds: Vec<u32>,
    #[arg(long, value_enum)]
    pub bqsr_mode: Option<BqsrModeArg>,
    #[arg(long)]
    pub known_sites: Vec<String>,
    #[arg(long)]
    pub bqsr_min_mean_coverage: Option<f64>,
    #[arg(long)]
    pub bqsr_min_breadth_1x: Option<f64>,
    #[arg(long)]
    pub haplogroup_panel: Option<String>,
    #[arg(long)]
    pub haplogroup_min_coverage: Option<f64>,
    #[arg(long)]
    pub kinship_panel: Option<String>,
    #[arg(long)]
    pub min_overlap_snps: Option<u32>,
    #[arg(long)]
    pub caller: Option<String>,
    #[arg(long)]
    pub min_posterior: Option<f64>,
    #[arg(long)]
    pub min_call_rate: Option<f64>,
    #[arg(long)]
    pub gc_bias_correction: bool,
    #[arg(long)]
    pub map_bias_correction: bool,
    #[arg(long)]
    pub authenticity_mode: Option<String>,
    #[arg(long)]
    pub aligner_preset: Option<String>,
    #[arg(long)]
    pub rg_id: Option<String>,
    #[arg(long)]
    pub rg_sm: Option<String>,
    #[arg(long)]
    pub rg_pl: Option<String>,
    #[arg(long)]
    pub rg_lb: Option<String>,
    #[arg(long, value_enum)]
    pub rg_policy: Option<ReadGroupPolicyArg>,
    #[arg(long)]
    pub build_reference_indices: bool,
    #[arg(long, value_name = "PATH")]
    pub params_json: Option<PathBuf>,
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long, help = "Allow planned/out-of-scope stages in planning")]
    pub allow_planned: bool,
}

#[derive(Debug, clap::Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum BamCommand {
    Run(BamRunArgs),
    ListStages,
    Explain {
        #[arg(long, value_enum)]
        stage: BamStageArg,
    },
}

#[derive(Debug, clap::Subcommand)]
pub enum BenchBamCommand {
    Stage(BenchBamStageArgs),
    Pipeline(BenchBamPipelineArgs),
}

#[derive(Debug, Args, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct BenchBamStageArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long, value_enum)]
    pub stage: BamStageArg,
    #[arg(long)]
    pub bam: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, value_delimiter = ',')]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
    #[arg(long)]
    pub allow_silver: bool,
    #[arg(long)]
    pub allow_experimental: bool,
    #[arg(long, default_value_t = 1)]
    pub replicates: u32,
    #[arg(long, default_value_t = 1)]
    pub jobs: u32,
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long, help = "Allow planned/out-of-scope stages in planning")]
    pub allow_planned: bool,
}

#[derive(Debug, Args, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct BenchBamPipelineArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long, default_value = "bam-to-bam__default__v1")]
    pub profile: String,
    #[arg(long)]
    pub bam: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, value_delimiter = ',')]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
    #[arg(long)]
    pub allow_silver: bool,
    #[arg(long)]
    pub allow_experimental: bool,
    #[arg(long, default_value_t = 1)]
    pub replicates: u32,
    #[arg(long, default_value_t = 1)]
    pub jobs: u32,
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long, help = "Allow planned/out-of-scope stages in planning")]
    pub allow_planned: bool,
}
