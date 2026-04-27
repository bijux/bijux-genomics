use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use toml::Value;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("workspace root"))
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
fn command_inventory_points_to_stage_authority() {
    let doc = read_doc(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs").join("COMMANDS.md"));
    assert!(
        doc.contains("## Runtime Commands\nNone."),
        "COMMANDS.md must state that the planner exposes no runtime commands",
    );
    for authority in ["bijux_dna_domain_fastq::STAGES", "src/tool_adapters/"] {
        assert!(doc.contains(authority), "COMMANDS.md must point to command authority {authority}",);
    }
}

#[test]
fn dependencies_doc_describes_forbidden_runtime_boundary() {
    let doc =
        read_doc(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs").join("DEPENDENCIES.md"));
    assert!(
        doc.contains("runner, engine, CLI, API, database, environment"),
        "DEPENDENCIES.md must explain that runtime and application crates stay downstream",
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
    let doc = read_doc(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs").join("COMMANDS.md"));
    assert!(
        doc.contains("`fastq.infer_asvs`"),
        "COMMANDS.md must list admitted FASTQ stages including infer_asvs",
    );
}

#[test]
fn add_tool_doc_refuses_manual_mapping_updates() {
    let doc = read_doc(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs").join("INDEX.md"));
    assert!(
        doc.contains("Do not maintain manual stage-to-tool matrices in Markdown"),
        "INDEX.md must route contributors to domain/test authority instead of manual mapping docs",
    );
}
