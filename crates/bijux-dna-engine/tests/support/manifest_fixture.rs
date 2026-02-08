use std::path::Path;

use anyhow::Result;
use bijux_dna_core::contract::ContractVersion;
use bijux_dna_core::prelude::input_assessment::FastqLayout;
use bijux_dna_runtime::run_layout::{write_manifest, RunArtifactEntry, RunLayout, RunManifest};

pub fn layout_tree_text(root: &Path) -> Result<String> {
    let mut entries = Vec::new();
    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.file_name().and_then(|name| name.to_str()) == Some("execution_record.json") {
            continue;
        }
        let rel = path.strip_prefix(root).unwrap_or(path);
        let hash = bijux_dna_infra::hash_file_sha256(path)?;
        entries.push(format!("{}\t{}", rel.display(), hash));
    }
    entries.sort();
    Ok(entries.join("\n"))
}

pub fn write_manifest_hash(
    layout: &RunLayout,
    graph: &bijux_dna_core::contract::execution::ExecutionGraph,
    output_path: &Path,
) -> Result<String> {
    let artifact_hash = bijux_dna_infra::hash_file_sha256(output_path)?;
    let manifest = RunManifest {
        schema_version: "bijux.run_manifest.v1".to_string(),
        contract_version: ContractVersion::v1(),
        run_id: "run-1".to_string(),
        started_at: "2024-01-01T00:00:00Z".to_string(),
        finished_at: "2024-01-01T00:00:01Z".to_string(),
        pipeline: graph.pipeline_id().as_str().to_string(),
        graph_hash: graph.hash()?,
        cache_key: None,
        layout: FastqLayout::SingleEnd,
        stages: Vec::new(),
        tool_invocations: Vec::new(),
        artifacts: vec![RunArtifactEntry {
            name: "output".to_string(),
            path: output_path.to_path_buf(),
            sha256: artifact_hash,
        }],
    };
    write_manifest(layout, &manifest)?;
    Ok(manifest.hash()?)
}
