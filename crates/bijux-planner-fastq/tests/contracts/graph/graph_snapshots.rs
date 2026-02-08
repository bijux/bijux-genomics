use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_core::contract::PlanPolicy;
use bijux_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_planner_fastq::stage_api::default_tool_for_stage;
use bijux_planner_fastq::{
    default_pipeline_spec, plan_fastq_to_fastq__default__v1, DefaultPipelineOptions,
    FastqPipelineInputs,
};

fn snapshot_name(group: &str, name: &str) -> String {
    format!("bijux-planner-fastq__{group}__{name}")
}

fn tool_for_stage(stage: &str) -> ToolExecutionSpecV1 {
    let stage_id = bijux_core::ids::StageId::new(stage);
    let tool_id = default_tool_for_stage(&stage_id).unwrap_or_else(|| "planner".to_string());
    ToolExecutionSpecV1 {
        tool_id: ToolId::new(tool_id),
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

/// Snapshot locks graph structure for the default FASTQ pipeline.
#[test]
fn fastq_default_pipeline_graph_is_pure() -> anyhow::Result<()> {
    let pipeline = default_pipeline_spec(DefaultPipelineOptions::default());
    let tools = pipeline
        .stages
        .iter()
        .map(|stage| tool_for_stage(stage))
        .collect::<Vec<_>>();
    let temp = bijux_infra::temp_dir("fastq-graph-snapshot")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let inputs = FastqPipelineInputs {
        policy: PlanPolicy::PreferAccuracy,
        tools,
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1: r1.clone(),
        r2: None,
        out_dir: temp.path().join("out"),
        tool_reasons: None,
    };

    let graph = plan_fastq_to_fastq__default__v1(&inputs, DefaultPipelineOptions::default())?;
    let json = serde_json::to_value(&graph)?;
    let json = bijux_core::contract::canonical::canonicalize_truth_json(&json);
    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots"));
    settings.set_prepend_module_to_snapshot(false);
    settings.add_filter(temp.path().to_str().unwrap_or_default(), "<temp>");
    settings.bind(|| {
        let name = snapshot_name("contracts", "fastq_default_graph");
        insta::assert_json_snapshot!(name, bijux_testkit::snapshot_normalize_json(&json));
    });
    Ok(())
}
