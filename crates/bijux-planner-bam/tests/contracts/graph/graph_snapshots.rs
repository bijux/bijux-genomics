use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_core::contract::PlanPolicy;
use bijux_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_domain_bam::BamStage;
use bijux_planner_bam::{plan_bam_to_bam__adna_shotgun__v1, BamPipelineInputs};

fn snapshot_name(group: &str, name: &str) -> String {
    format!("bijux-planner-bam__{group}__{name}")
}

fn dummy_tool(stage: &str) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId::new(format!("tool.{stage}")),
        tool_version: "0.0.0".to_string(),
        image: ContainerImageRefV1 {
            image: "bijux/dummy:latest".to_string(),
            digest: None,
        },
        command: CommandSpecV1 {
            template: vec!["echo".to_string(), stage.to_string()],
        },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
    }
}

/// Snapshot locks graph structure for the default aDNA shotgun pipeline.
#[test]
fn bam_adna_shotgun_graph_is_pure() -> anyhow::Result<()> {
    let mut tool_specs = BTreeMap::new();
    for stage in BamStage::all() {
        tool_specs.insert(stage.as_str().to_string(), dummy_tool(stage.as_str()));
    }

    let temp = bijux_infra::temp_dir("bam-graph-snapshot")?;
    let bam = temp.path().join("sample.bam");
    std::fs::write(&bam, b"")?;

    let inputs = BamPipelineInputs {
        policy: PlanPolicy::PreferAccuracy,
        tool_specs,
        params_overrides: BTreeMap::new(),
        bam: bam.clone(),
        bam_index: None,
        reference: None,
        sample_id: Some("sample".to_string()),
        out_dir: temp.path().join("out"),
    };

    let graph = plan_bam_to_bam__adna_shotgun__v1(&inputs)?;
    let json = serde_json::to_value(&graph)?;
    let json = bijux_core::contract::canonical::canonicalize_truth_json(&json);
    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots"));
    settings.set_prepend_module_to_snapshot(false);
    settings.add_filter(temp.path().to_str().unwrap_or_default(), "<temp>");
    settings.bind(|| {
        let name = snapshot_name("contracts", "bam_adna_shotgun_graph");
        insta::assert_json_snapshot!(name, json);
    });
    Ok(())
}
