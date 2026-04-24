use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use toml::Value;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn read_doc(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn parse_toml(path: &Path) -> Value {
    let raw = read_doc(path);
    raw.parse::<Value>().unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}

#[test]
fn stage_mapping_points_to_manifest_authorities() {
    let doc =
        read_doc(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs").join("STAGE_MAPPING.md"));
    assert!(
        doc.contains("intentionally not a manual stage-to-tool matrix"),
        "STAGE_MAPPING.md must refuse manual stage/tool tables",
    );
    for authority in [
        "domain/fastq/index.yaml",
        "domain/fastq/execution_support.yaml",
        "domain/fastq/stages/*.yaml",
        "domain/fastq/tools/*.yaml",
        "src/tool_adapters/fastq.rs",
    ] {
        assert!(
            doc.contains(authority),
            "STAGE_MAPPING.md must point to manifest authority {authority}",
        );
    }
}

#[test]
fn tool_selection_doc_describes_closed_runtime_boundary() {
    let doc =
        read_doc(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs").join("TOOL_SELECTION.md"));
    assert!(
        doc.contains("publish only the\nclosed FASTQ execution surface")
            || doc.contains("publish only the closed FASTQ execution surface"),
        "TOOL_SELECTION.md must explain that generated configs expose only closed runtime support",
    );
}

#[test]
fn ci_stage_catalog_excludes_declared_only_fastq_stages() {
    let stages = parse_toml(&workspace_root().join("configs/ci/stages/stages.toml"));
    let stage_ids = stages
        .get("stages")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|stage| stage.get("id").and_then(Value::as_str))
        .collect::<BTreeSet<_>>();
    assert!(
        stage_ids.contains("fastq.infer_asvs"),
        "the curated CI stage catalog must publish governed FASTQ stages once their runtime support is closed",
    );
}

#[test]
fn ci_tool_registry_excludes_unpublished_fastq_tools() {
    let registry = parse_toml(&workspace_root().join("configs/ci/registry/tool_registry.toml"));
    let tool_ids = registry
        .get("tools")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|tool| tool.get("id").and_then(Value::as_str))
        .collect::<BTreeSet<_>>();
    for tool_id in ["diamond", "dustmasker", "seqfu", "seqpurge"] {
        assert!(
            !tool_ids.contains(tool_id),
            "planned FASTQ tool {tool_id} must stay out of the governed CI runtime registry",
        );
    }
    for tool_id in ["alientrimmer", "fastx_clipper", "leehom", "skewer"] {
        assert!(
            tool_ids.contains(tool_id),
            "governed FASTQ tool {tool_id} must stay in the curated CI runtime registry once its containerized runtime is closed",
        );
    }
    assert!(
        tool_ids.contains("fastq_scan"),
        "fastq_scan must stay in the governed CI runtime registry once its containerized validate runtime is closed"
    );
}

#[test]
fn stage_mapping_documents_declared_only_infer_asvs() {
    let doc =
        read_doc(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs").join("STAGE_MAPPING.md"));
    assert!(
        doc.contains("`fastq.infer_asvs`") && doc.contains("governed `dada2` runtime contract"),
        "STAGE_MAPPING.md must explain the admitted infer_asvs runtime boundary",
    );
}

#[test]
fn add_tool_doc_refuses_manual_mapping_updates() {
    let doc = read_doc(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs").join("ADD_TOOL.md"));
    assert!(
        doc.contains("Do not maintain manual stage-to-tool tables in docs"),
        "ADD_TOOL.md must route contributors to manifest SSOT instead of manual mapping docs",
    );
}
