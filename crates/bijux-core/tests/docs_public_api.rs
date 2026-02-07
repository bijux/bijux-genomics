use std::fs;
use std::path::PathBuf;

fn read_doc(path: &PathBuf) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| panic!("read {}", path.display()))
}

#[test]
fn docs_cover_public_api_modules() {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs");
    let public_api = read_doc(&base.join("PUBLIC_API.md"));
    let index = read_doc(&base.join("INDEX.md"));
    let contracts = read_doc(&base.join("CONTRACTS.md"));
    let ssot = read_doc(&base.join("SSOT.md"));

    let docs = format!("{}\n{}\n{}", index, contracts, ssot).to_lowercase();

    let modules: Vec<String> = public_api
        .lines()
        .filter_map(|line| line.strip_prefix("- "))
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    assert!(!modules.is_empty(), "PUBLIC_API.md must list modules");

    for module in modules {
        assert!(
            docs.contains(&module),
            "Docs must mention public module `{}` in INDEX/CONTRACTS/SSOT",
            module
        );
    }
}

#[test]
fn minimal_contract_example_builds() {
    use bijux_core::contract::execution::PlanPolicy;
    use bijux_core::contract::execution::{ExecutionEdge, ExecutionGraph, ExecutionStep};
    use bijux_core::contract::{ArtifactRole, StageIO, ToolConstraints};
    use bijux_core::foundation::{CommandSpecV1, ContainerImageRefV1};
    use bijux_core::ids::{ArtifactId, PipelineId, StageId, StepId};
    use std::path::PathBuf;

    let step = ExecutionStep {
        step_id: StepId::from_static("step.a"),
        stage_id: StageId::from_static("stage.a"),
        command: CommandSpecV1 {
            template: vec!["echo".to_string(), "ok".to_string()],
        },
        image: ContainerImageRefV1 {
            image: "example/image".to_string(),
            digest: Some("sha256:deadbeef".to_string()),
        },
        resources: ToolConstraints::default(),
        io: StageIO {
            inputs: vec![],
            outputs: vec![bijux_core::contract::execution::ArtifactSpec::required(
                ArtifactId::from_static("artifact.a"),
                PathBuf::from("artifact.txt"),
                ArtifactRole::ReportJson,
            )],
        },
        out_dir: PathBuf::from("out"),
        aux_images: Default::default(),
        expected_artifact_ids: vec![ArtifactId::from_static("artifact.a")],
        metrics_schema_ids: vec![],
    };

    let graph = ExecutionGraph::new(
        PipelineId::from_static("pipeline.a").as_str(),
        "planner.v1",
        PlanPolicy::default(),
        vec![step],
        vec![],
    )
    .expect("build graph");

    assert_eq!(graph.steps().len(), 1);
    assert!(graph.edges().is_empty());
}
