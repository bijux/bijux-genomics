use std::path::PathBuf;

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct BenchBamStageArgs {
    pub sample_id: String,
    pub stage: bijux_domain_bam::BamStage,
    pub bam: PathBuf,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub allow_silver: bool,
    pub allow_experimental: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub dry_run: bool,
}

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct BenchBamPipelineArgs {
    pub profile: String,
    pub sample_id: String,
    pub bam: PathBuf,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub allow_silver: bool,
    pub allow_experimental: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub dry_run: bool,
}

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct BamRunArgs {
    pub stage: bijux_domain_bam::BamStage,
    pub profile: String,
    pub sample_id: Option<String>,
    pub r1: Option<PathBuf>,
    pub r2: Option<PathBuf>,
    pub bam: PathBuf,
    pub out: PathBuf,
    pub tool: Option<String>,
    pub bai: Option<PathBuf>,
    pub reference: Option<PathBuf>,
    pub regions: Option<String>,
    pub udg_model: Option<String>,
    pub pmd_threshold_5p: Option<f64>,
    pub pmd_threshold_3p: Option<f64>,
    pub trim_5p: Option<u32>,
    pub trim_3p: Option<u32>,
    pub contamination_scope: Option<String>,
    pub contamination_panel: Vec<String>,
    pub contamination_prior: Option<f64>,
    pub sex_specific_contamination: bool,
    pub contamination_assumptions: Option<String>,
    pub expected_sex: Option<String>,
    pub sex_method: String,
    pub min_mapq: Option<u32>,
    pub min_length: Option<u32>,
    pub include_flags: Vec<String>,
    pub exclude_flags: Vec<String>,
    pub remove_duplicates: bool,
    pub base_quality_threshold: Option<u8>,
    pub optical_duplicates: Option<String>,
    pub umi_policy: Option<String>,
    pub duplicate_action: Option<String>,
    pub complexity_min_reads: Option<u32>,
    pub complexity_projection_points: Vec<u64>,
    pub depth_thresholds: Vec<u32>,
    pub bqsr_mode: Option<String>,
    pub known_sites: Vec<String>,
    pub bqsr_min_mean_coverage: Option<f64>,
    pub bqsr_min_breadth_1x: Option<f64>,
    pub haplogroup_panel: Option<String>,
    pub haplogroup_min_coverage: Option<f64>,
    pub kinship_panel: Option<String>,
    pub min_overlap_snps: Option<u32>,
    pub caller: Option<String>,
    pub min_posterior: Option<f64>,
    pub min_call_rate: Option<f64>,
    pub gc_bias_correction: bool,
    pub map_bias_correction: bool,
    pub authenticity_mode: Option<String>,
    pub aligner_preset: Option<String>,
    pub rg_id: Option<String>,
    pub rg_sm: Option<String>,
    pub rg_pl: Option<String>,
    pub rg_lb: Option<String>,
    pub rg_policy: Option<String>,
    pub build_reference_indices: bool,
    pub params_json: Option<String>,
    pub dry_run: bool,
}

#[derive(Debug, Clone)]
pub struct FastqCrossArgs {
    pub sample_id: Option<String>,
    pub r1: Option<PathBuf>,
    pub r2: Option<PathBuf>,
    pub alignment_bam: Option<PathBuf>,
    pub alignment_bai: Option<PathBuf>,
    pub alignment_reference: Option<PathBuf>,
    pub alignment_rg_policy: Option<String>,
    pub alignment_meta: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RunRequest {
    pub profile_id: String,
    pub domain: bijux_pipelines::Domain,
    pub run_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct RunResult {
    pub run_dir: PathBuf,
    pub profile_id: String,
}

#[derive(Debug, Clone)]
pub struct PlanRunRequest {
    pub run_spec: bijux_core::RunSpec,
    pub profile: bijux_core::Profile,
    pub run_id: bijux_core::RunId,
}

#[derive(Debug, Clone)]
pub struct PlanRunResult {
    pub plan: bijux_core::ExecutionPlan,
}

#[derive(Debug, Clone)]
pub struct ExecuteRunRequest {
    pub plan: bijux_core::StagePlanV1,
    pub runner: bijux_env_runtime::api::RunnerKind,
}

#[derive(Debug, Clone)]
pub struct ExecuteRunResult;

#[derive(Debug, Clone)]
pub struct RenderReportRequest {
    pub base_dir: PathBuf,
    pub facts_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct RenderReportResult {
    pub report_path: PathBuf,
}
