/// Snapshot intent: verifies stable, reviewed output for this contract.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bijux_core::contract::PlanPolicy;
use bijux_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_domain_bam::BamStage;
use bijux_planner_bam::{
    plan_bam_to_bam__adna_capture__v1, plan_bam_to_bam__adna_shotgun__v1, plan_stage,
    BamPipelineInputs, BamPlanConfig, BamPlanner, StagePlanRequest,
};

fn snapshot_name(group: &str, name: &str) -> String {
    format!("bijux-planner-bam__{group}__{name}")
}

fn snapshot_settings(temp_path: Option<&Path>) -> insta::Settings {
    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots"));
    settings.set_prepend_module_to_snapshot(false);
    if let Some(path) = temp_path {
        settings.add_filter(path.to_str().unwrap_or_default(), "<temp>");
    }
    settings
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
    for stage_id in bijux_planner_bam::pipeline_id_catalog(profile_id) {
        let stage = BamStage::try_from(stage_id.as_str()).expect("stage id");
        let tool_id = bijux_planner_bam::stage_api::default_tool_for_stage(stage);
        specs.insert(stage_id, dummy_tool(&tool_id));
    }
    specs
}

#[test]
fn bam_plan_snapshot() {
    let _guard = snapshot_settings(None).bind_to_scope();
    let fixtures = Path::new("tests/fixtures/plan_inputs/default");
    let tool_id = bijux_planner_bam::stage_api::default_tool_for_stage(BamStage::Align);
    let tool_align = dummy_tool(&tool_id);
    let stage_plan = plan_stage(StagePlanRequest {
        stage_id: BamStage::Align.as_str(),
        tool: &tool_align,
        out_dir: Path::new("out"),
        bam: Some(&fixtures.join("sample.bam")),
        bam_index: Some(&fixtures.join("sample.bam.bai")),
        r1: Some(&fixtures.join("reads_R1.fastq.gz")),
        r2: Some(&fixtures.join("reads_R2.fastq.gz")),
        reference: Some(&fixtures.join("reference.fasta")),
        sample_id: Some("sample"),
        params: None,
    })
    .expect("plan stage");
    let config = BamPlanConfig {
        pipeline_id: "bam-to-bam__adna_shotgun__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        stages: vec![stage_plan],
    };
    let plan = BamPlanner::plan(&config).expect("plan");
    let name = snapshot_name("contracts", "bam_plan_snapshot");
    insta::assert_json_snapshot!(name, bijux_testkit::snapshot_normalize_json(&plan));
}

#[test]
fn bam_adna_shotgun_plan_snapshot() -> anyhow::Result<()> {
    let temp = bijux_infra::temp_dir("bam-adna-shotgun-plan")?;
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
    };
    let plan = plan_bam_to_bam__adna_shotgun__v1(&inputs)?;
    let name = snapshot_name("contracts", "bam_adna_shotgun_plan");
    let _guard = snapshot_settings(Some(temp.path())).bind_to_scope();
    insta::assert_json_snapshot!(name, bijux_testkit::snapshot_normalize_json(&plan));
    Ok(())
}

#[test]
fn bam_adna_capture_plan_snapshot() -> anyhow::Result<()> {
    let temp = bijux_infra::temp_dir("bam-adna-capture-plan")?;
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
    };
    let plan = plan_bam_to_bam__adna_capture__v1(&inputs)?;
    let name = snapshot_name("contracts", "bam_adna_capture_plan");
    let _guard = snapshot_settings(Some(temp.path())).bind_to_scope();
    insta::assert_json_snapshot!(name, bijux_testkit::snapshot_normalize_json(&plan));
    Ok(())
}
