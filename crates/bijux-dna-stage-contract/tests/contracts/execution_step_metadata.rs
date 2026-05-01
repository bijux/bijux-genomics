use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_dna_core::contract::{ArtifactRef, ArtifactRole, StageIO, ToolConstraints};
use bijux_dna_core::ids::{ArtifactId, StageId, StageVersion, ToolId};
use bijux_dna_core::prelude::{CommandSpecV1, ContainerImageRefV1, StepId};
use bijux_dna_stage_contract::{
    execution_step_from_stage_plan_with_step_id, PlanDecisionReason, StagePlanV1,
};

fn trim_plan() -> StagePlanV1 {
    StagePlanV1 {
        stage_id: StageId::from_static("fastq.trim_reads"),
        stage_instance_id: None,
        stage_version: StageVersion(1),
        tool_id: ToolId::from_static("fastp"),
        tool_version: "test".to_string(),
        image: ContainerImageRefV1 { image: "bijux/test".to_string(), digest: None },
        command: CommandSpecV1 { template: vec!["fastp".to_string()] },
        resources: ToolConstraints::default(),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::from_static("reads_r1"),
                PathBuf::from("reads_R1.fastq.gz"),
                ArtifactRole::Reads,
            )],
            outputs: vec![
                ArtifactRef::required(
                    ArtifactId::from_static("trimmed_reads_r1"),
                    PathBuf::from("trimmed_R1.fastq.gz"),
                    ArtifactRole::TrimmedReads,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("trimmed_reads_r2"),
                    PathBuf::from("trimmed_R2.fastq.gz"),
                    ArtifactRole::TrimmedReads,
                ),
            ],
        },
        out_dir: PathBuf::from("out"),
        params: serde_json::json!({}),
        effective_params: serde_json::json!({}),
        aux_images: BTreeMap::new(),
        operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
        canonical_contract: None,
        provenance: None,
        reason: PlanDecisionReason::default(),
    }
}

#[test]
fn execution_steps_inherit_expected_artifacts_and_metrics_schema() {
    let step = execution_step_from_stage_plan_with_step_id(
        &trim_plan(),
        StepId::new("fastq.trim_reads.tool.fastp"),
    );
    assert_eq!(
        step.expected_artifact_ids
            .iter()
            .map(bijux_dna_core::contract::ArtifactId::as_str)
            .collect::<Vec<_>>(),
        vec!["trimmed_reads_r1", "trimmed_reads_r2"]
    );
    assert_eq!(step.metrics_schema_ids, vec!["fastq_trim_reads_v2".to_string()]);
}
