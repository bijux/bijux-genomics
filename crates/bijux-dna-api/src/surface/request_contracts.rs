use std::path::PathBuf;

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
/// Benchmark a single BAM stage.
///
/// Stability: v1 (stable).
pub struct BenchBamStageArgs {
    pub sample_id: String,
    pub stage: bijux_dna_planner_bam::stage_api::BamStage,
    pub bam: PathBuf,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub allow_silver: bool,
    pub allow_experimental: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub dry_run: bool,
    pub allow_planned: bool,
}

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
/// Benchmark a full BAM pipeline.
///
/// Stability: v1 (stable).
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
    pub allow_planned: bool,
}

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
/// Run a BAM pipeline with explicit tool selection.
///
/// Stability: v1 (stable).
pub struct BamRunArgs {
    pub stage: bijux_dna_planner_bam::stage_api::BamStage,
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
    pub alignment_sensitivity_profile: Option<String>,
    pub alignment_seed_length: Option<u32>,
    pub rg_id: Option<String>,
    pub rg_sm: Option<String>,
    pub rg_pl: Option<String>,
    pub rg_lb: Option<String>,
    pub rg_pu: Option<String>,
    pub lane_id: Option<String>,
    pub run_id: Option<String>,
    pub subject_id: Option<String>,
    pub cohort_id: Option<String>,
    pub rg_policy: Option<String>,
    pub build_reference_indices: bool,
    pub params_json: Option<String>,
    pub dry_run: bool,
    pub allow_planned: bool,
}

#[derive(Debug, Clone)]
/// Plan a cross-domain FASTQ→BAM run.
///
/// Stability: v1 (stable).
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
/// Execute a pipeline run request.
///
/// Stability: v1 (stable).
pub struct RunRequest {
    pub profile_id: String,
    pub domain: bijux_dna_pipelines::Domain,
    pub run_dir: PathBuf,
}

#[derive(Debug, Clone)]
/// Result for a completed run execution.
///
/// Stability: v1 (stable).
pub struct RunResult {
    pub run_dir: PathBuf,
    pub profile_id: String,
}

#[derive(Debug, Clone)]
/// Plan a pipeline execution.
///
/// Stability: v1 (stable).
pub struct PlanRunRequest {
    pub run_spec: bijux_dna_core::contract::RunSpec,
    pub profile: bijux_dna_core::contract::Profile,
    pub run_id: bijux_dna_core::contract::RunId,
}

#[derive(Debug, Clone)]
/// Planned stages and inferred outputs for a pipeline.
///
/// Stability: v1 (stable).
pub struct PlanRunResult {
    pub plan: bijux_dna_stage_contract::RunExecutionPlan,
}

#[derive(Debug, Clone)]
/// Execute a plan run request.
///
/// Stability: v1 (stable).
pub struct ExecuteRunRequest {
    pub plan: bijux_dna_stage_contract::StagePlanV1,
    pub runner: bijux_dna_environment::api::RuntimeKind,
}

#[derive(Debug, Clone)]
/// Result of executing a plan run request.
///
/// Stability: v1 (stable).
pub struct ExecuteRunResult;

#[derive(Debug, Clone)]
/// Render a report bundle from facts rows.
///
/// Stability: v1 (stable).
pub struct RenderReportRequest {
    pub base_dir: PathBuf,
    pub facts_path: PathBuf,
}

#[derive(Debug, Clone)]
/// Output paths from report rendering.
///
/// Stability: v1 (stable).
pub struct RenderReportResult {
    pub report_path: PathBuf,
    pub evidence_bundle_path: PathBuf,
}

#[derive(Debug, Clone)]
/// Run status snapshot.
///
/// Stability: v1 (stable).
pub struct RunStatus {
    pub run_dir: PathBuf,
    pub manifest_path: Option<PathBuf>,
    pub report_path: Option<PathBuf>,
    pub evidence_bundle_path: Option<PathBuf>,
    pub evidence_verification_path: Option<PathBuf>,
    pub artifact_inventory_path: Option<PathBuf>,
    pub artifact_inventory_text_path: Option<PathBuf>,
    pub replay_manifest_path: Option<PathBuf>,
    pub hash_ledger_path: Option<PathBuf>,
    pub run_summary_text_path: Option<PathBuf>,
    pub run_state_path: Option<PathBuf>,
    pub runtime_policy_path: Option<PathBuf>,
    pub executor_descriptor_path: Option<PathBuf>,
    pub backend_descriptor_path: Option<PathBuf>,
    pub scheduling_decision_path: Option<PathBuf>,
    pub queue_state_path: Option<PathBuf>,
    pub lease_path: Option<PathBuf>,
    pub control_state_path: Option<PathBuf>,
    pub health_report_path: Option<PathBuf>,
    pub slurm_submission_path: Option<PathBuf>,
    pub checkpoint_path: Option<PathBuf>,
    pub failure_path: Option<PathBuf>,
    pub correlation_id: Option<String>,
    pub mode: Option<bijux_dna_runtime::run_layout::RunExecutionModeV1>,
    pub state: Option<bijux_dna_runtime::run_layout::RunLifecycleStateV1>,
    pub has_failures: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Canonical run-control response.
///
/// Stability: v1 (stable).
pub struct RunControlResponse {
    pub control_state_path: PathBuf,
    pub queue_state_path: Option<PathBuf>,
    pub state: bijux_dna_runtime::run_layout::RunControlStateV1,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Canonical operator-health response.
///
/// Stability: v1 (stable).
pub struct OperatorHealthResponse {
    pub health_report_path: PathBuf,
    pub report: bijux_dna_runtime::run_layout::OperatorHealthReportV1,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Canonical plan request.
///
/// Stability: v1 (stable).
pub struct PlanRequest {
    pub graph: bijux_dna_core::contract::ExecutionGraph,
    pub profile_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_manifest: Option<bijux_dna_core::contract::WorkflowManifestV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stage_plans: Vec<bijux_dna_stage_contract::StagePlanV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameter_traces: Vec<bijux_dna_core::contract::ParameterResolutionTraceV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub planner_refusals: Vec<bijux_dna_core::contract::PlannerRefusalRecordV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub planner_warnings: Vec<bijux_dna_core::contract::PlannerWarningRecordV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compare_against: Option<bijux_dna_core::contract::PlanManifestV1>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Canonical plan response.
///
/// Stability: v1 (stable).
pub struct PlanResponse {
    pub graph: bijux_dna_core::contract::ExecutionGraph,
    pub graph_hash: String,
    pub manifest: serde_json::Value,
    pub workflow_manifest: bijux_dna_core::contract::WorkflowManifestV1,
    pub plan_manifest: bijux_dna_core::contract::PlanManifestV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_diff: Option<bijux_dna_core::contract::PlanManifestDiffV1>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Canonical execute request.
///
/// Stability: v1 (stable).
pub struct ExecuteRequest {
    pub graph: bijux_dna_core::contract::ExecutionGraph,
    pub runner: bijux_dna_environment::api::RuntimeKind,
    pub run_dir: PathBuf,
    #[serde(default)]
    pub mode: bijux_dna_runtime::run_layout::RunExecutionModeV1,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Canonical execute response.
///
/// Stability: v1 (stable).
pub struct ExecuteResponse {
    pub run_id: String,
    pub correlation_id: String,
    pub manifest_path: PathBuf,
    pub run_state_path: PathBuf,
    pub runtime_policy_path: PathBuf,
    pub executor_descriptor_path: PathBuf,
    pub backend_descriptor_path: PathBuf,
    pub scheduling_decision_path: PathBuf,
    pub queue_state_path: PathBuf,
    pub lease_path: PathBuf,
    pub control_state_path: PathBuf,
    pub health_report_path: PathBuf,
    pub slurm_submission_path: Option<PathBuf>,
    pub checkpoint_path: PathBuf,
    pub failure_path: Option<PathBuf>,
    pub mode: bijux_dna_runtime::run_layout::RunExecutionModeV1,
    pub state: bijux_dna_runtime::run_layout::RunLifecycleStateV1,
    pub report_path: Option<PathBuf>,
    pub evidence_bundle_path: PathBuf,
    pub evidence_verification_path: PathBuf,
    pub artifact_inventory_path: PathBuf,
    pub replay_manifest_path: PathBuf,
    pub hash_ledger_path: PathBuf,
    pub run_summary_text_path: PathBuf,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Canonical dry-run request.
///
/// Stability: v1 (stable).
pub struct DryRunRequest {
    pub graph: bijux_dna_core::contract::ExecutionGraph,
    pub run_dir: PathBuf,
    pub profile_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Canonical dry-run response.
///
/// Stability: v1 (stable).
pub struct DryRunResponse {
    pub graph_path: PathBuf,
    pub manifest_path: PathBuf,
    pub run_summary_path: PathBuf,
    pub run_summary_text_path: PathBuf,
    pub run_state_path: PathBuf,
    pub runtime_policy_path: PathBuf,
    pub executor_descriptor_path: PathBuf,
    pub backend_descriptor_path: PathBuf,
    pub scheduling_decision_path: PathBuf,
    pub queue_state_path: PathBuf,
    pub lease_path: PathBuf,
    pub control_state_path: PathBuf,
    pub health_report_path: PathBuf,
    pub checkpoint_path: PathBuf,
    pub mode: bijux_dna_runtime::run_layout::RunExecutionModeV1,
    pub state: bijux_dna_runtime::run_layout::RunLifecycleStateV1,
    pub evidence_bundle_path: PathBuf,
    pub evidence_verification_path: PathBuf,
    pub artifact_inventory_path: PathBuf,
    pub replay_manifest_path: PathBuf,
    pub hash_ledger_path: PathBuf,
    pub slurm_submission_path: Option<PathBuf>,
    pub correlation_id: String,
}
