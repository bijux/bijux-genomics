use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::preprocess::{
    LibraryDamageTreatment, PreprocessEffectiveParams,
};
use bijux_dna_domain_fastq::params::PairedMode;
use bijux_dna_domain_fastq::FastqPipelineMode;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = StageId::from_static("fastq.preprocess");
pub const STAGE_VERSION: StageVersion = StageVersion(1);

#[derive(Debug, Clone)]
pub struct PreprocessPlan {
    pub r1: std::path::PathBuf,
    pub r2: Option<std::path::PathBuf>,
    pub stages: Vec<String>,
    pub enable_contaminant_removal: bool,
    pub pipeline_mode: FastqPipelineMode,
}

/// # Errors
/// Returns an error if the synthetic preprocess summary plan cannot be materialized.
pub fn plan_preprocess_stage(
    plan: &PreprocessPlan,
    tool: &ToolExecutionSpecV1,
) -> Result<StagePlanV1> {
    let paired_mode = if plan.r2.is_some() { PairedMode::PairedEnd } else { PairedMode::SingleEnd };
    let effective_params = PreprocessEffectiveParams {
        pipeline_mode: plan.pipeline_mode,
        enable_contaminant_removal: plan.enable_contaminant_removal,
        paired_mode,
        library_declared_paired: plan.r2.is_some(),
        library_damage_treatment: LibraryDamageTreatment::Unknown,
        stages: plan.stages.clone(),
        threads: tool.resources.threads,
    };
    let out_dir = plan.r1.parent().unwrap_or_else(|| std::path::Path::new(".")).join("out");
    Ok(StagePlanV1 {
        stage_id: STAGE_ID.clone(),
        stage_instance_id: Some(crate::tool_adapters::default_stage_instance_id(
            &STAGE_ID,
            &tool.tool_id,
        )),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: bijux_dna_core::prelude::CommandSpecV1 { template: tool.command.template.clone() },
        resources: tool.resources.clone(),
        io: StageIO {
            inputs: {
                let mut inputs = Vec::new();
                inputs.push(ArtifactRef::required(
                    ArtifactId::from_static("reads_r1"),
                    plan.r1.clone(),
                    ArtifactRole::Reads,
                ));
                if let Some(r2) = plan.r2.as_ref() {
                    inputs.push(ArtifactRef::required(
                        ArtifactId::from_static("reads_r2"),
                        r2.clone(),
                        ArtifactRole::Reads,
                    ));
                }
                inputs
            },
            outputs: Vec::new(),
        },
        out_dir,
        params: serde_json::json!({
            "r1": plan.r1,
            "r2": plan.r2,
            "stages": plan.stages,
        }),
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize preprocess effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
        canonical_contract: None,
        provenance: None,
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}
