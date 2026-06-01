use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bijux_dna_core::contract::PlanPolicy;

use bijux_dna_stage_contract::StagePlanV1;

pub mod stage_api {
    pub use crate::local_readiness::{local_align_plan, local_contamination_plan};
    pub use crate::local_smoke::{
        local_authenticity_smoke_plans, local_complexity_smoke_plans, local_coverage_smoke_plans,
        local_damage_smoke_plans, local_duplication_metrics_smoke_plans,
        local_endogenous_content_smoke_plans, local_filter_smoke_plans, local_gc_bias_smoke_plans,
        local_insert_size_smoke_plans, local_length_filter_smoke_plans,
        local_mapping_summary_smoke_plans, local_mapq_filter_smoke_plans,
        local_markdup_smoke_plans, local_overlap_correction_smoke_plans, local_qc_pre_smoke_plans,
        local_sex_smoke_plans, local_validate_smoke_plans, LocalAuthenticitySmokeCasePlan,
        LocalComplexitySmokeCasePlan, LocalCoverageSmokeCasePlan, LocalCoverageSmokeExpectedRow,
        LocalDamageSmokeCasePlan, LocalDuplicationMetricsSmokeCasePlan,
        LocalEndogenousContentSmokeCasePlan, LocalFilterSmokeCasePlan, LocalGcBiasSmokeCasePlan,
        LocalGcBiasSmokeExpectedRow, LocalInsertSizeSmokeCasePlan, LocalLengthFilterSmokeCasePlan,
        LocalMappingSummarySmokeCasePlan, LocalMapqFilterSmokeCasePlan, LocalMarkdupSmokeCasePlan,
        LocalOverlapCorrectionSmokeCasePlan, LocalQcPreSmokeCasePlan, LocalSexSmokeCasePlan,
        LocalValidateSmokeCasePlan,
    };
    pub use crate::report_stage::report_stage_step;
    pub use crate::selection::{allowed_tools_for_stage, default_tool_for_stage};
    pub use crate::stages::stage_registry;
    pub use crate::{plan_stage, StagePlanRequest};
    pub use bijux_dna_stages_bam::stage_specs::*;
}

#[derive(Debug, Clone)]
pub struct BamPlanConfig {
    pub pipeline_id: String,
    pub policy: PlanPolicy,
    pub stages: Vec<StagePlanV1>,
}

#[derive(Debug, Clone)]
pub struct BamPipelineInputs {
    pub policy: PlanPolicy,
    pub tool_specs: BTreeMap<String, bijux_dna_core::contract::ToolExecutionSpecV1>,
    pub params_overrides: BTreeMap<String, serde_json::Value>,
    pub bam: PathBuf,
    pub bam_index: Option<PathBuf>,
    pub reference: Option<PathBuf>,
    pub sample_id: Option<String>,
    pub out_dir: PathBuf,
    pub allow_planned: bool,
}

pub struct StagePlanRequest<'a> {
    pub stage_id: &'a str,
    pub tool: &'a bijux_dna_core::contract::ToolExecutionSpecV1,
    pub out_dir: &'a Path,
    pub bam: Option<&'a Path>,
    pub bam_index: Option<&'a Path>,
    pub r1: Option<&'a Path>,
    pub r2: Option<&'a Path>,
    pub reference: Option<&'a Path>,
    pub sample_id: Option<&'a str>,
    pub params: Option<&'a serde_json::Value>,
}
