use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_dna_core::contract::{ArtifactRef, ArtifactRole, StageIO, ToolConstraints};
use bijux_dna_core::ids::{ArtifactId, StageId, StageVersion, ToolId};
use bijux_dna_core::prelude::{CommandSpecV1, ContainerImageRefV1};
use bijux_dna_stage_contract::StagePlugin;
use bijux_dna_stage_contract::{PlanDecisionReason, StagePlanV1};
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
        operating_mode: bijux_dna_core::contract::StageOperatingMode::default(),
        aux_images: BTreeMap::new(),
        canonical_contract: None,
        provenance: None,
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

#[test]
fn bam_stage_plugin_rejects_empty_command_templates() {
    let plugin = BamStagePlugin;
    let mut plan = stage_plan("bam.mapping_summary");
    plan.command.template.clear();

    let error = match plugin.materialize(&plan) {
        Ok(_) => panic!("empty command templates must fail"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("empty command template"));
}

#[test]
fn bam_stage_plugin_rejects_blank_command_template_arguments() {
    let plugin = BamStagePlugin;
    let mut plan = stage_plan("bam.mapping_summary");
    plan.command.template = vec!["samtools".to_string(), "   ".to_string()];

    let error = match plugin.materialize(&plan) {
        Ok(_) => panic!("blank command template arguments must fail"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("blank command template argument"));
}

#[test]
fn bam_stage_plugin_preserves_reported_output_artifacts() -> anyhow::Result<()> {
    let plugin = BamStagePlugin;
    let plan = stage_plan("bam.mapping_summary");
    let outputs = plan.io.outputs.clone();

    let parsed = plugin.parse_outputs(&plan, &outputs)?;

    assert_eq!(parsed.artifacts, outputs);
    Ok(())
}

#[test]
fn bam_stage_plugin_reads_plan_out_dir_when_outputs_are_empty() -> anyhow::Result<()> {
    let plugin = BamStagePlugin;
    let temp = bijux_dna_infra::temp_dir("bijux-bam-plugin-out-dir")?;
    let mut plan = stage_plan("bam.mapping_summary");
    plan.out_dir = temp.path().to_path_buf();
    bijux_dna_infra::write_bytes(
        plan.out_dir.join("flagstat.txt"),
        include_bytes!("../fixtures/observer/default/flagstat.txt"),
    )?;

    let parsed = plugin.parse_outputs(&plan, &[])?;

    assert_eq!(parsed.metrics.metrics["alignment"]["total"], 10);
    Ok(())
}

#[test]
fn bam_stage_plugin_input_fingerprint_is_stable_for_reordered_inputs() -> anyhow::Result<()> {
    let plugin = BamStagePlugin;
    let temp = bijux_dna_infra::temp_dir("bijux-bam-plugin-input-order")?;
    let input_a = temp.path().join("a.bam");
    let input_b = temp.path().join("b.bam");
    bijux_dna_infra::write_bytes(&input_a, b"input-a")?;
    bijux_dna_infra::write_bytes(&input_b, b"input-b")?;

    let input_ref_a = ArtifactRef::required(ArtifactId::new("input_a"), input_a, ArtifactRole::Bam);
    let input_ref_b = ArtifactRef::required(ArtifactId::new("input_b"), input_b, ArtifactRole::Bam);
    let mut first_plan = stage_plan("bam.mapping_summary");
    first_plan.io.inputs = vec![input_ref_a.clone(), input_ref_b.clone()];
    let mut second_plan = stage_plan("bam.mapping_summary");
    second_plan.io.inputs = vec![input_ref_b, input_ref_a];

    let first = plugin.parse_outputs(&first_plan, &[])?;
    let second = plugin.parse_outputs(&second_plan, &[])?;

    assert_eq!(first.metrics.input_hashes, second.metrics.input_hashes);
    assert_eq!(first.metrics.input_fingerprint, second.metrics.input_fingerprint);
    Ok(())
}

#[test]
fn bam_stage_plugin_metric_discovery_is_stable_for_reordered_outputs() -> anyhow::Result<()> {
    let plugin = BamStagePlugin;
    let temp = bijux_dna_infra::temp_dir("bijux-bam-plugin-output-order")?;
    let metrics_dir = temp.path().join("metrics");
    let reports_dir = temp.path().join("reports");
    bijux_dna_infra::ensure_dir(&metrics_dir)?;
    bijux_dna_infra::ensure_dir(&reports_dir)?;
    bijux_dna_infra::write_bytes(
        metrics_dir.join("flagstat.txt"),
        include_bytes!("../fixtures/observer/default/flagstat.txt"),
    )?;

    let metrics_output = ArtifactRef::required(
        ArtifactId::new("flagstat"),
        metrics_dir.join("flagstat.txt"),
        ArtifactRole::ReportJson,
    );
    let report_output = ArtifactRef::required(
        ArtifactId::new("summary"),
        reports_dir.join("summary.json"),
        ArtifactRole::ReportJson,
    );

    let mut plan = stage_plan("bam.mapping_summary");
    plan.out_dir = temp.path().join("unused");

    let first = plugin.parse_outputs(&plan, &[metrics_output.clone(), report_output.clone()])?;
    let second = plugin.parse_outputs(&plan, &[report_output, metrics_output])?;

    assert_eq!(first.metrics.metrics["alignment"]["total"], 10);
    assert_eq!(first.metrics.metrics, second.metrics.metrics);
    Ok(())
}

#[test]
fn bam_stage_plugin_trims_tool_version_in_metrics_envelope() -> anyhow::Result<()> {
    let plugin = BamStagePlugin;
    let mut plan = stage_plan("bam.mapping_summary");
    plan.tool_version = " 1.17 ".to_string();

    let parsed = plugin.parse_outputs(&plan, &[])?;

    assert_eq!(parsed.metrics.tool_version, "1.17");
    Ok(())
}
