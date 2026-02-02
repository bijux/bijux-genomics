// imports provided by core parser module

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum BamStageArg {
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
    pub fn stage(self) -> bijux_domain_bam::BamStage {
        match self {
            BamStageArg::Validate => bijux_domain_bam::BamStage::Validate,
            BamStageArg::QcPre => bijux_domain_bam::BamStage::QcPre,
            BamStageArg::Filter => bijux_domain_bam::BamStage::Filter,
            BamStageArg::Markdup => bijux_domain_bam::BamStage::Markdup,
            BamStageArg::Complexity => bijux_domain_bam::BamStage::Complexity,
            BamStageArg::Coverage => bijux_domain_bam::BamStage::Coverage,
            BamStageArg::Damage => bijux_domain_bam::BamStage::Damage,
            BamStageArg::Authenticity => bijux_domain_bam::BamStage::Authenticity,
            BamStageArg::Contamination => bijux_domain_bam::BamStage::Contamination,
            BamStageArg::Sex => bijux_domain_bam::BamStage::Sex,
            BamStageArg::BiasMitigation => bijux_domain_bam::BamStage::BiasMitigation,
            BamStageArg::Recalibration => bijux_domain_bam::BamStage::Recalibration,
            BamStageArg::Haplogroups => bijux_domain_bam::BamStage::Haplogroups,
            BamStageArg::Genotyping => bijux_domain_bam::BamStage::Genotyping,
            BamStageArg::Kinship => bijux_domain_bam::BamStage::Kinship,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum UdgModelArg {
    NonUdg,
    HalfUdg,
    Udg,
}

impl From<UdgModelArg> for bijux_domain_bam::UdgModel {
    fn from(value: UdgModelArg) -> Self {
        match value {
            UdgModelArg::NonUdg => bijux_domain_bam::UdgModel::NonUdg,
            UdgModelArg::HalfUdg => bijux_domain_bam::UdgModel::HalfUdg,
            UdgModelArg::Udg => bijux_domain_bam::UdgModel::Udg,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ContaminationScopeArg {
    Mito,
    Nuclear,
    Both,
}

impl From<ContaminationScopeArg> for bijux_domain_bam::ContaminationScope {
    fn from(value: ContaminationScopeArg) -> Self {
        match value {
            ContaminationScopeArg::Mito => bijux_domain_bam::ContaminationScope::Mito,
            ContaminationScopeArg::Nuclear => bijux_domain_bam::ContaminationScope::Nuclear,
            ContaminationScopeArg::Both => bijux_domain_bam::ContaminationScope::Both,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum BqsrModeArg {
    Standard,
    Skip,
    EmitOnly,
}

impl From<BqsrModeArg> for bijux_domain_bam::BqsrMode {
    fn from(value: BqsrModeArg) -> Self {
        match value {
            BqsrModeArg::Standard => bijux_domain_bam::BqsrMode::Standard,
            BqsrModeArg::Skip => bijux_domain_bam::BqsrMode::Skip,
            BqsrModeArg::EmitOnly => bijux_domain_bam::BqsrMode::EmitOnly,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ExpectedSexArg {
    Xx,
    Xy,
    Unknown,
}

impl From<ExpectedSexArg> for bijux_domain_bam::ExpectedSex {
    fn from(value: ExpectedSexArg) -> Self {
        match value {
            ExpectedSexArg::Xx => bijux_domain_bam::ExpectedSex::XX,
            ExpectedSexArg::Xy => bijux_domain_bam::ExpectedSex::XY,
            ExpectedSexArg::Unknown => bijux_domain_bam::ExpectedSex::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OpticalDuplicatePolicyArg {
    None,
    MarkOnly,
    Remove,
}

impl From<OpticalDuplicatePolicyArg> for bijux_domain_bam::OpticalDuplicatePolicy {
    fn from(value: OpticalDuplicatePolicyArg) -> Self {
        match value {
            OpticalDuplicatePolicyArg::None => bijux_domain_bam::OpticalDuplicatePolicy::None,
            OpticalDuplicatePolicyArg::MarkOnly => {
                bijux_domain_bam::OpticalDuplicatePolicy::MarkOnly
            }
            OpticalDuplicatePolicyArg::Remove => bijux_domain_bam::OpticalDuplicatePolicy::Remove,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum UmiPolicyArg {
    Ignore,
    UseTag,
    Collapse,
}

impl From<UmiPolicyArg> for bijux_domain_bam::UmiPolicy {
    fn from(value: UmiPolicyArg) -> Self {
        match value {
            UmiPolicyArg::Ignore => bijux_domain_bam::UmiPolicy::Ignore,
            UmiPolicyArg::UseTag => bijux_domain_bam::UmiPolicy::UseTag,
            UmiPolicyArg::Collapse => bijux_domain_bam::UmiPolicy::Collapse,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum DuplicateActionArg {
    Mark,
    Remove,
}

impl From<DuplicateActionArg> for bijux_domain_bam::DuplicateAction {
    fn from(value: DuplicateActionArg) -> Self {
        match value {
            DuplicateActionArg::Mark => bijux_domain_bam::DuplicateAction::Mark,
            DuplicateActionArg::Remove => bijux_domain_bam::DuplicateAction::Remove,
        }
    }
}

#[derive(Debug, Args, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct BamRunArgs {
    #[arg(long, value_enum, default_value_t = BamStageArg::Validate)]
    pub stage: BamStageArg,
    #[arg(long, default_value = "default")]
    pub profile: String,
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
    #[arg(long, value_name = "PATH")]
    pub params_json: Option<PathBuf>,
    #[arg(long)]
    pub dry_run: bool,
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
}
