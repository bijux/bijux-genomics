use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_dna_core::contract::{ArtifactRef, ArtifactRole, StageIO, ToolConstraints};
use bijux_dna_core::ids::{ArtifactId, StageId, StageVersion, ToolId};
use bijux_dna_core::prelude::{CommandSpecV1, ContainerImageRefV1};
use bijux_dna_stage_contract::{PlanDecisionReason, StagePlanV1};
use bijux_dna_stage_contract::StagePlugin;
use bijux_dna_stages_bam::BamStagePlugin;

fn stage_plan(stage_id: &str) -> StagePlanV1 {
    StagePlanV1 {
        stage_id: StageId::new(stage_id),
        stage_instance_id: None,
        stage_version: StageVersion(1),
        tool_id: ToolId::new("samtools"),
        tool_version: "1.17".to_string(),
        image: ContainerImageRefV1 { image: "samtools".to_string(), digest: None },
        command: CommandSpecV1 { template: vec!["samtools".to_string(), "flagstat".to_string()] },
        resources: ToolConstraints::default(),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::new("input_bam"),
                PathBuf::from("input.bam"),
                ArtifactRole::Bam,
            )],
            outputs: vec![ArtifactRef::required(
                ArtifactId::new("flagstat"),
                PathBuf::from("flagstat.txt"),
                ArtifactRole::ReportJson,
            )],
        },
        out_dir: PathBuf::from("out"),
        params: serde_json::json!({}),
        effective_params: serde_json::json!({}),
        aux_images: BTreeMap::new(),
        reason: PlanDecisionReason::default(),
    }
}

#[test]
fn bam_stage_plugin_handles_only_registered_bam_stage_ids() {
    let plugin = BamStagePlugin;

    assert!(plugin.handles_stage("bam.align"));
    assert!(!plugin.handles_stage("bam.not_registered"));
    assert!(!plugin.handles_stage("fastq.validate_reads"));
}

#[test]
fn bam_stage_plugin_rejects_materializing_unsupported_stage_ids() {
    let plugin = BamStagePlugin;
    let plan = stage_plan("bam.not_registered");

    let error = match plugin.materialize(&plan) {
        Ok(_) => panic!("unsupported BAM stages must fail"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("unsupported BAM stage bam.not_registered"));
}

#[test]
fn bam_stage_plugin_rejects_parsing_unsupported_stage_ids() {
    let plugin = BamStagePlugin;
    let plan = stage_plan("bam.not_registered");

    let error = match plugin.parse_outputs(&plan, &[]) {
        Ok(_) => panic!("unsupported BAM stages must fail"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("unsupported BAM stage bam.not_registered"));
}
