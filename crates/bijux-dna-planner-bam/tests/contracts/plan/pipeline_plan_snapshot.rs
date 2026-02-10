/// Snapshot intent: verifies stable, reviewed output for this contract.
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bijux_dna_core::contract::PlanPolicy;
use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_dna_domain_bam::BamStage;
use bijux_dna_planner_bam::BamPipelineInputs;

fn snapshot_name(group: &str, name: &str) -> String {
    format!("bijux-dna-planner-bam__{group}__{name}")
}

fn dummy_tool(tool: &str) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId::new(tool),
        tool_version: "0.7.17".to_string(),
        image: ContainerImageRefV1 {
            image: "bijux/bwa".to_string(),
            digest: Some("sha256:bwa".to_string()),
        },
        command: CommandSpecV1 {
            template: vec!["bwa".to_string()],
        },
        resources: ToolConstraints {
            runtime: "short".to_string(),
            mem_gb: 2,
            tmp_gb: 1,
            threads: 2,
        },
    }
}

fn tool_specs_for_profile(profile_id: &str) -> BTreeMap<String, ToolExecutionSpecV1> {
    let mut specs = BTreeMap::new();
    for stage_id in bijux_dna_planner_bam::pipeline_id_catalog(profile_id) {
        let stage = BamStage::try_from(stage_id.as_str()).expect("stage id");
        let tool_id = bijux_dna_planner_bam::stage_api::default_tool_for_stage(stage);
        specs.insert(stage_id, dummy_tool(tool_id.as_str()));
    }
    specs
}

fn assert_snapshot(name: &str, payload: &serde_json::Value, temp_path: &Path) -> Result<()> {
    let name = snapshot_name("contracts", name);
    let payload = serde_json::to_string_pretty(payload)?;
    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots"));
    settings.set_prepend_module_to_snapshot(false);
    settings.add_filter(temp_path.to_str().unwrap_or_default(), "<temp>");
    settings.bind(|| {
        insta::assert_snapshot!(name, bijux_dna_testkit::snapshot_normalize_text(&payload));
    });
    Ok(())
}

#[test]
fn pipeline_plan_snapshots_are_stable() -> Result<()> {
    let temp = bijux_dna_infra::temp_dir("bam-adna-shotgun-plan")?;
    let bam = temp.path().join("sample.bam");
    std::fs::write(&bam, b"")?;
    let inputs = BamPipelineInputs {
        policy: PlanPolicy::PreferAccuracy,
        tool_specs: tool_specs_for_profile("bam-to-bam__adna_shotgun__v1"),
        params_overrides: BTreeMap::new(),
        bam: bam.clone(),
        bam_index: None,
        reference: None,
        sample_id: Some("sample".to_string()),
        out_dir: temp.path().join("out"),
        allow_planned: false,
    };
    let payload = serde_json::to_value(bijux_dna_planner_bam::plan_bam_to_bam__adna_shotgun__v1(
        &inputs,
    )?)?;
    assert_snapshot(
        "pipeline__bam__bam-to-bam__adna_shotgun__v1",
        &payload,
        temp.path(),
    )?;

    let temp = bijux_dna_infra::temp_dir("bam-adna-capture-plan")?;
    let bam = temp.path().join("sample.bam");
    std::fs::write(&bam, b"")?;
    let inputs = BamPipelineInputs {
        policy: PlanPolicy::PreferAccuracy,
        tool_specs: tool_specs_for_profile("bam-to-bam__adna_capture__v1"),
        params_overrides: BTreeMap::new(),
        bam: bam.clone(),
        bam_index: None,
        reference: None,
        sample_id: Some("sample".to_string()),
        out_dir: temp.path().join("out"),
        allow_planned: false,
    };
    let payload = serde_json::to_value(bijux_dna_planner_bam::plan_bam_to_bam__adna_capture__v1(
        &inputs,
    )?)?;
    assert_snapshot(
        "pipeline__bam__bam-to-bam__adna_capture__v1",
        &payload,
        temp.path(),
    )?;
    Ok(())
}
