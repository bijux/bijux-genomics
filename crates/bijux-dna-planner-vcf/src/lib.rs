use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::contract::{ExecutionEdge, ExecutionGraph, PlanPolicy};
use bijux_dna_core::ids::{ArtifactId, StageId, StageVersion, StepId, ToolId};
use bijux_dna_core::prelude::{
    ArtifactRole, ArtifactSpec, CommandSpecV1, ContainerImageRefV1, StageIO, ToolConstraints,
};
use bijux_dna_domain_vcf::{
    params::{VcfCallParams, VcfFilterParams, VcfStatsParams},
    VcfStage,
};
use bijux_dna_stage_contract::StagePlanV1;

pub const PLANNER_VERSION: &str = "bijux-dna-planner-vcf.v1";

#[derive(Debug, Clone)]
pub struct VcfPipelineInputs {
    pub policy: PlanPolicy,
    pub vcf: std::path::PathBuf,
    pub out_dir: std::path::PathBuf,
}

fn default_tool(stage: VcfStage) -> &'static str {
    match stage {
        VcfStage::Call => "bcftools",
        VcfStage::Filter => "bcftools",
        VcfStage::Stats => "bcftools",
    }
}

fn stage_plan(stage: VcfStage, input_vcf: &Path, out_dir: &Path) -> StagePlanV1 {
    let stage_id = stage.as_str();
    let output_name = match stage {
        VcfStage::Call => "called_vcf",
        VcfStage::Filter => "filtered_vcf",
        VcfStage::Stats => "stats_json",
    };
    let output_path = match stage {
        VcfStage::Stats => out_dir.join("stats.json"),
        _ => out_dir.join(format!("{}.vcf.gz", output_name)),
    };

    let params = match stage {
        VcfStage::Call => serde_json::to_value(VcfCallParams::default()).unwrap_or_default(),
        VcfStage::Filter => serde_json::to_value(VcfFilterParams::default()).unwrap_or_default(),
        VcfStage::Stats => serde_json::to_value(VcfStatsParams::default()).unwrap_or_default(),
    };

    StagePlanV1 {
        stage_id: StageId::new(stage_id.to_string()),
        stage_version: StageVersion(1),
        tool_id: ToolId::new(default_tool(stage)),
        tool_version: "1.20".to_string(),
        image: ContainerImageRefV1 {
            image: "quay.io/biocontainers/bcftools:1.20--h8b25389_0".to_string(),
            digest: Some(
                "sha256:67f54df47f501f6ddef08e3b9ad89cf693952f9a89de0d74df6e39fce15f1ff6"
                    .to_string(),
            ),
        },
        command: CommandSpecV1 {
            template: vec![default_tool(stage).to_string(), "--help".to_string()],
        },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 2,
            tmp_gb: 2,
            threads: 2,
        },
        io: StageIO {
            inputs: vec![ArtifactSpec::required(
                ArtifactId::new("vcf"),
                input_vcf.to_path_buf(),
                ArtifactRole::Reads,
            )],
            outputs: vec![ArtifactSpec::required(
                ArtifactId::new(output_name),
                output_path,
                if matches!(stage, VcfStage::Stats) {
                    ArtifactRole::MetricsJson
                } else {
                    ArtifactRole::Reads
                },
            )],
        },
        out_dir: out_dir.to_path_buf(),
        params: params.clone(),
        effective_params: params,
        aux_images: BTreeMap::new(),
        reason: Default::default(),
    }
}

/// # Errors
/// Returns an error when no stage plans can be generated.
pub fn plan_vcf_minimal(inputs: &VcfPipelineInputs) -> Result<ExecutionGraph> {
    let stages = [VcfStage::Call, VcfStage::Filter, VcfStage::Stats];
    let plans = stages
        .iter()
        .map(|stage| stage_plan(*stage, &inputs.vcf, &inputs.out_dir))
        .collect::<Vec<_>>();
    if plans.is_empty() {
        return Err(anyhow!("no VCF stage plans generated"));
    }

    let steps = plans
        .iter()
        .map(bijux_dna_stage_contract::execution_step_from_stage_plan)
        .collect::<Vec<_>>();
    let edges = vec![
        ExecutionEdge::new(StepId::new("vcf.call"), StepId::new("vcf.filter")),
        ExecutionEdge::new(StepId::new("vcf.filter"), StepId::new("vcf.stats")),
    ];
    Ok(ExecutionGraph::new(
        "vcf-to-vcf__minimal__v1".to_string(),
        PLANNER_VERSION,
        inputs.policy,
        steps,
        edges,
    )?)
}
