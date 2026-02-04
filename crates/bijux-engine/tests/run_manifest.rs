use std::fs;

use bijux_engine::primitives::{prepare_tool_run_dirs, write_run_manifest, RunArtifactInput};

#[test]
fn run_manifest_includes_telemetry_and_facts() -> anyhow::Result<()> {
    let dir = bijux_infra::temp_dir("bijux")?;
    let out_dir = dir.path().join("out");
    bijux_infra::ensure_dir(&out_dir)?;
    let run_dirs = prepare_tool_run_dirs(&out_dir, "stage", "tool")?;
    let adapter_bank = out_dir.join("adapter_bank.yaml");
    bijux_infra::write_bytes(&adapter_bank, "bank")?;
    bijux_infra::write_bytes(&run_dirs.manifest_path, "{}")?;
    bijux_infra::write_bytes(&run_dirs.metrics_path, "{}")?;
    bijux_infra::write_bytes(&run_dirs.retention_report_path, "{}")?;
    write_run_manifest(
        &run_dirs,
        "stage",
        "tool",
        &adapter_bank,
        &[] as &[RunArtifactInput],
    )?;
    let raw = fs::read_to_string(&run_dirs.run_manifest_path)?;
    let manifest: serde_json::Value = serde_json::from_str(&raw)?;
    assert!(manifest.get("telemetry").is_some());
    assert!(manifest
        .get("dashboard")
        .and_then(|v| v.get("facts_jsonl"))
        .is_some());
    Ok(())
}
