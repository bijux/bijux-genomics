use std::fs;
use std::path::{Path, PathBuf};

use bijux_core::contract::execution::ExecutionGraph;
use bijux_core::contract::RunRecordV1;
use bijux_core::metrics::ToolInvocationV1;

fn collect_json_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if !root.exists() {
        return files;
    }
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    files.push(path);
                }
            }
        }
    }
    files
}

fn parse_core_fixture(schema: &str, value: serde_json::Value, path: &Path) {
    match schema {
        "bijux.execution_graph.v1" => {
            let graph: ExecutionGraph = serde_json::from_value(value)
                .unwrap_or_else(|err| panic!("{path:?}: ExecutionGraph parse failed: {err}"));
            graph
                .validate_strict()
                .unwrap_or_else(|err| panic!("{path:?}: ExecutionGraph validate failed: {err}"));
        }
        "bijux.run_record.v1" => {
            let _: RunRecordV1 = serde_json::from_value(value)
                .unwrap_or_else(|err| panic!("{path:?}: RunRecordV1 parse failed: {err}"));
        }
        "bijux.tool_invocation.v1" => {
            let _: ToolInvocationV1 = serde_json::from_value(value)
                .unwrap_or_else(|err| panic!("{path:?}: ToolInvocationV1 parse failed: {err}"));
        }
        _ => {
            // Not a core schema; ignore.
        }
    }
}

#[test]
fn contract_fixtures_from_other_crates_parse() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("resolve repo root");

    let fixture_roots = [
        repo_root.join("crates/bijux-planner-fastq/tests/fixtures"),
        repo_root.join("crates/bijux-planner-bam/tests/fixtures"),
        repo_root.join("crates/bijux-stage-contract/tests/fixtures"),
        repo_root.join("crates/bijux-stages-fastq/tests/fixtures"),
        repo_root.join("crates/bijux-stages-bam/tests/fixtures"),
        repo_root.join("crates/bijux-runtime/tests/fixtures"),
        repo_root.join("crates/bijux-analyze/tests/fixtures"),
    ];

    let mut parsed = 0usize;
    for root in fixture_roots {
        for path in collect_json_files(&root) {
            let raw = match fs::read_to_string(&path) {
                Ok(text) => text,
                Err(_) => continue,
            };
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) else {
                continue;
            };
            let schema = value
                .get("schema_version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            if let Some(schema) = schema {
                let before = parsed;
                parse_core_fixture(&schema, value, &path);
                if matches!(
                    schema.as_str(),
                    "bijux.execution_graph.v1" | "bijux.run_record.v1" | "bijux.tool_invocation.v1"
                ) {
                    parsed = before + 1;
                }
            }
        }
    }

    assert!(
        parsed > 0,
        "expected to parse at least one core fixture from other crates"
    );
}
