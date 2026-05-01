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
    let contracts = read_doc(&base.join("CONTRACTS.md"));
    let contract_map = read_doc(&base.join("CONTRACT_MAP.md"));
    let commands = read_doc(&base.join("COMMANDS.md"));

    let docs = format!("{contracts}\n{contract_map}\n{commands}").to_lowercase();

    let modules: Vec<String> = public_api
        .lines()
        .filter_map(public_module_from_doc_line)
        .map(|s| s.to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    assert!(!modules.is_empty(), "PUBLIC_API.md must list modules");

    for module in modules {
        assert!(
            docs.contains(&module),
            "Docs must mention public module `{module}` in CONTRACTS/CONTRACT_MAP/COMMANDS"
        );
    }
}

#[test]
fn commands_doc_is_managed_operation_inventory() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let commands = read_doc(&root.join("docs/COMMANDS.md"));
    let readme = read_doc(&root.join("README.md"));

    let operations: Vec<String> = commands.lines().filter_map(operation_from_table_row).collect();
    let expected = [
        "canonicalize-json",
        "canonicalize-parameters-json",
        "canonicalize-truth-json",
        "canonical-json-bytes",
        "params-hash",
        "parameters-fingerprint",
        "input-fingerprint",
        "run-id-from-hashes",
        "parse-pipeline-id",
        "validate-pipeline-id",
        "parse-stage-id",
        "validate-stage-id",
        "parse-tool-id",
        "validate-tool-id",
        "validate-artifact-id",
        "validate-profile-id",
        "discover-fastq-files",
        "detect-fastq-path",
        "detect-gzip-path",
        "assess-input-dir",
        "write-input-assessment",
        "validate-execution-graph",
        "hash-execution-graph",
        "normalize-execution-graph",
        "topological-step-ids",
        "validate-execution-outputs",
        "query-run-index",
        "build-run-dir",
        "select-stage",
        "objective-spec",
        "parse-metric-id",
        "parse-derived-metric-id",
        "validate-metric-id",
        "validate-derived-metric-id",
        "metrics-schema-for-stage",
    ];

    assert_eq!(
        operations, expected,
        "COMMANDS.md must list the managed operation inventory in stable order"
    );

    for operation in expected {
        assert!(
            readme.contains(&format!("`{operation}`")),
            "README.md must reference managed operation `{operation}`"
        );
    }
}

#[test]
fn readme_documents_the_exact_docs_allowance() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let docs_dir = root.join("docs");
    let readme = read_doc(&root.join("README.md"));
    let mut actual = fs::read_dir(&docs_dir)
        .unwrap_or_else(|err| panic!("read {}: {err}", docs_dir.display()))
        .map(|entry| {
            entry
                .unwrap_or_else(|err| panic!("read entry in {}: {err}", docs_dir.display()))
                .file_name()
                .to_string_lossy()
                .to_string()
        })
        .collect::<Vec<_>>();
    actual.sort();

    let declared = readme
        .lines()
        .filter_map(|line| line.trim().strip_prefix("- `docs/"))
        .filter_map(|line| line.strip_suffix('`'))
        .map(str::to_string)
        .collect::<Vec<_>>();

    assert_eq!(actual.len(), 10, "core docs allowance is exactly 10 files");
    assert_eq!(declared, actual, "README.md documentation list must match docs/ exactly");
}

fn public_module_from_doc_line(line: &str) -> Option<String> {
    let line = line.trim();
    if let Some(rest) = line.strip_prefix("- `") {
        return rest.strip_suffix('`').map(str::to_string);
    }
    let cells = markdown_table_cells(line);
    if cells.len() >= 2 && cells[0].starts_with('`') && cells[0].ends_with('`') {
        return Some(cells[0].trim_matches('`').to_string());
    }
    None
}

fn operation_from_table_row(line: &str) -> Option<String> {
    let cells = markdown_table_cells(line);
    if cells.len() < 3 {
        return None;
    }
    let operation = cells[0].trim_matches('`');
    if operation.is_empty() || operation == "Operation" || operation.chars().all(|ch| ch == '-') {
        return None;
    }
    Some(operation.to_string())
}

fn markdown_table_cells(line: &str) -> Vec<&str> {
    let line = line.trim();
    if !line.starts_with('|') || !line.ends_with('|') {
        return Vec::new();
    }
    line.trim_matches('|').split('|').map(str::trim).collect()
}

#[test]
fn minimal_contract_example_builds() {
    use bijux_dna_core::contract::execution::PlanPolicy;
    use bijux_dna_core::contract::execution::{ExecutionGraph, ExecutionStep};
    use bijux_dna_core::contract::{ArtifactRole, StageIO, ToolConstraints};
    use bijux_dna_core::ids::{ArtifactId, PipelineId, StageId, StepId};
    use bijux_dna_core::prelude::{CommandSpecV1, ContainerImageRefV1};
    use std::path::PathBuf;

    let step = ExecutionStep {
        step_id: StepId::from_static("step.a"),
        stage_id: StageId::from_static("stage.a"),
        command: CommandSpecV1 { template: vec!["echo".to_string(), "ok".to_string()] },
        image: ContainerImageRefV1 {
            image: "example/image".to_string(),
            digest: Some("sha256:deadbeef".to_string()),
        },
        resources: ToolConstraints::default(),
        io: StageIO {
            inputs: vec![bijux_dna_core::contract::execution::ArtifactSpec::required(
                ArtifactId::from_static("artifact.in"),
                PathBuf::from("input.txt"),
                ArtifactRole::Reads,
            )],
            outputs: vec![bijux_dna_core::contract::execution::ArtifactSpec::required(
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
