use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[test]
fn public_api_doc_matches_runtime_root_surface() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let docs = read(root.join("docs/PUBLIC_API.md"));
    let lib = read(root.join("src/lib.rs"));

    assert_eq!(
        section_items(&docs, "Public Modules"),
        entries([
            "environment",
            "manifests",
            "observability",
            "provenance",
            "recording",
            "run",
            "run_layout",
            "runner",
            "telemetry",
        ]),
        "PUBLIC_API.md must list every public module from src/lib.rs"
    );

    for module in section_items(&docs, "Public Modules") {
        assert!(
            lib.contains(&format!("pub mod {module};")),
            "PUBLIC_API.md lists {module}, but src/lib.rs does not export it"
        );
    }

    for required_export in [
        "RunProvenanceV1",
        "RunContextV1",
        "TelemetryEventV1",
        "prepare_tool_run_dirs",
        "write_canonical_json",
        "write_profile_and_lock_manifests",
        "write_run_manifest",
        "create_run_layout",
        "write_run_state",
        "write_runtime_policy",
        "write_executor_descriptor",
        "write_checkpoint",
        "write_failure_record",
        "write_manifest",
        "RunManifest",
        "RunStageEntry",
        "ensure_stage_supported_by_runner",
        "Artifact",
        "Invocation",
        "Runner",
        "RunnerContractKind",
        "RunnerResult",
        "build_telemetry_adapter",
        "TelemetryAdapter",
        "TelemetrySpan",
    ] {
        assert!(
            docs.contains(required_export),
            "PUBLIC_API.md must document root export {required_export}"
        );
        assert!(
            lib.contains(required_export),
            "src/lib.rs must re-export documented root export {required_export}"
        );
    }
}

fn read(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();
    fs::read_to_string(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

fn section_items(docs: &str, heading: &str) -> BTreeSet<String> {
    let mut in_section = false;
    let mut items = BTreeSet::new();

    for line in docs.lines() {
        if line == format!("## {heading}") {
            in_section = true;
            continue;
        }
        if in_section && line.starts_with("## ") {
            break;
        }
        if in_section {
            if let Some(item) =
                line.trim().strip_prefix("- `").and_then(|item| item.strip_suffix('`'))
            {
                items.insert(item.to_string());
            }
        }
    }

    items
}

fn entries<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(str::to_string).collect()
}
