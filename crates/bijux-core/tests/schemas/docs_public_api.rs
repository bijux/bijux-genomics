use std::collections::BTreeMap;
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

    let docs = format!("{index}\n{contracts}\n{ssot}").to_lowercase();

    let modules: Vec<String> = public_api
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("- `") {
                return rest.strip_suffix('`').map(str::to_string);
            }
            if let Some(rest) = line.strip_prefix("- ") {
                return Some(rest.trim().to_string());
            }
            None
        })
        .map(|s| s.to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    assert!(!modules.is_empty(), "PUBLIC_API.md must list modules");

    for module in modules {
        assert!(
            docs.contains(&module),
            "Docs must mention public module `{module}` in INDEX/CONTRACTS/SSOT"
        );
    }
}

#[test]
fn minimal_contract_example_builds() {
    use bijux_core::contract::execution::PlanPolicy;
    use bijux_core::contract::execution::{ExecutionGraph, ExecutionStep};
    use bijux_core::contract::{ArtifactRole, StageIO, ToolConstraints};
    use bijux_core::ids::{ArtifactId, PipelineId, StageId, StepId};
    use bijux_core::prelude::{CommandSpecV1, ContainerImageRefV1};
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
            inputs: vec![bijux_core::contract::execution::ArtifactSpec::required(
                ArtifactId::from_static("artifact.in"),
                PathBuf::from("input.txt"),
                ArtifactRole::Reads,
            )],
            outputs: vec![bijux_core::contract::execution::ArtifactSpec::required(
                ArtifactId::from_static("artifact.a"),
                PathBuf::from("artifact.txt"),
                ArtifactRole::ReportJson,
            )],
        },
        out_dir: PathBuf::from("out"),
        aux_images: BTreeMap::default(),
        expected_artifact_ids: vec![ArtifactId::from_static("artifact.a")],
        metrics_schema_ids: vec![],
    };

    let graph = ExecutionGraph::new(
        PipelineId::from_static("x-to-y__default__v1").as_str(),
        "planner.v1",
        PlanPolicy::default(),
        vec![step],
        vec![],
    )
    .unwrap_or_else(|err| panic!("build graph: {err}"));

    assert_eq!(graph.steps().len(), 1);
    assert!(graph.edges().is_empty());
}
