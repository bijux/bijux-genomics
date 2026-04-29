use anyhow::Result;
use bijux_dna_api::v1::api::status;

#[test]
fn status_discovers_evidence_bundle_and_correlation() -> Result<()> {
    let temp = tempfile::tempdir()?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("run_manifest.json"),
        &serde_json::json!({
            "schema_version": "bijux.run_manifest.v3",
            "run_id": "run-1",
            "correlation_id": "corr-run-1",
            "failures": []
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("evidence_bundle.json"),
        &serde_json::json!({
            "schema_version": "bijux.evidence_bundle.v1"
        }),
    )?;

    let snapshot = status(temp.path())?;
    assert_eq!(snapshot.correlation_id.as_deref(), Some("corr-run-1"));
    assert_eq!(
        snapshot
            .evidence_bundle_path
            .as_deref()
            .map(std::path::Path::file_name)
            .and_then(|value| value.and_then(|value| value.to_str())),
        Some("evidence_bundle.json")
    );
    Ok(())
}
