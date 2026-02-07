use std::fs;
use std::path::PathBuf;

use bijux_core::contract::ContractVersion;
use bijux_core::foundation::input_assessment::FastqLayout;
use bijux_runtime::recording::{prepare_tool_run_dirs, write_canonical_json, write_run_manifest, RunArtifactInput};
use bijux_runtime::{create_run_layout, write_manifest, RunManifest, RunProvenanceV1, RunStageEntry};

#[test]
fn reference_example_layout_manifest_record() -> anyhow::Result<()> {
    let base = std::env::temp_dir().join(format!("bijux-runtime-example-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&base)?;

    let (run_id, layout) = create_run_layout(&base)?;
    let manifest = RunManifest {
        schema_version: "bijux.run_manifest.v1".to_string(),
        contract_version: ContractVersion::v1(),
        run_id: run_id.clone(),
        started_at: "1970-01-01T00:00:00Z".to_string(),
        finished_at: "1970-01-01T00:00:01Z".to_string(),
        pipeline: "example".to_string(),
        graph_hash: "graph-hash".to_string(),
        cache_key: None,
        layout: FastqLayout::SingleEnd,
        stages: vec![RunStageEntry {
            stage_id: "stage.example".to_string(),
            tool_id: "tool.example".to_string(),
            execution_metrics_path: PathBuf::from("stages/example/execution.json"),
            domain_metrics_path: PathBuf::from("stages/example/metrics.json"),
            logs_dir: PathBuf::from("stages/example/logs"),
            outputs_dir: PathBuf::from("stages/example/outputs"),
            tool_invocation_path: PathBuf::from("stages/example/tool_invocation.json"),
        }],
        tool_invocations: Vec::new(),
        artifacts: Vec::new(),
    };
    write_manifest(&layout, &manifest)?;
    assert!(layout.manifest_path.exists());

    let tools_root = layout.run_dir.join("tools");
    let run_dirs = prepare_tool_run_dirs(&tools_root, "stage.example", "run-1")?;
    write_canonical_json(&run_dirs.manifest_path, &serde_json::json!({"ok": true}))?;
    write_canonical_json(&run_dirs.metrics_path, &serde_json::json!({"metrics": []}))?;

    let provenance = RunProvenanceV1 {
        schema_version: "bijux.run_provenance.v1".to_string(),
        tool_image_digest: Some("sha256:deadbeef".to_string()),
        tool_version: "1.0.0".to_string(),
        params_hash: "params-hash".to_string(),
        input_hashes: vec!["input-hash".to_string()],
        reference_genome: None,
        pipeline_id: "example".to_string(),
        git_commit: "deadbeef".to_string(),
        build_profile: "dev".to_string(),
        plan_hash: Some("graph-hash".to_string()),
    };

    write_run_manifest(
        &run_dirs,
        "stage.example",
        "tool.example",
        &provenance,
        None,
        &Vec::<RunArtifactInput>::new(),
    )?;
    assert!(run_dirs.run_manifest_path.exists());

    fs::remove_dir_all(&base)?;
    Ok(())
}
