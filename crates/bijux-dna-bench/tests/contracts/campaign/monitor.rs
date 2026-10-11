#![allow(clippy::expect_used)]
use super::*;

#[test]
fn submission_observations_reject_ambiguous_identity_and_unknown_versions() -> anyhow::Result<()> {
    let root = bijux_dna_infra::temp_dir("campaign-monitor-")?;
    let path = root.path().join("submission.json");
    for (body, reason) in [
        (serde_json::json!({"jobs":[{}, {}]}), "missing planned job identity"),
        (
            serde_json::json!({"jobs":[{"planned_job_id":"one"}, {"planned_job_id":"one"}]}),
            "duplicate submission observation",
        ),
        (
            serde_json::json!({"schema_version":"unsupported", "jobs":[]}),
            "unsupported campaign submission observation schema",
        ),
    ] {
        bijux_dna_infra::write_bytes(&path, serde_json::to_vec(&body)?)?;
        let error =
            submission_jobs_from_manifest(&path).expect_err("must reject ambiguous observations");
        assert!(error.to_string().contains(reason), "{error:#}");
    }
    for versioned in [false, true] {
        let mut body = serde_json::json!({"jobs":[{"planned_job_id":"one", "scheduler_job_id":"42", "unrelated_metadata":"retained upstream"}]});
        if versioned {
            body["schema_version"] = CAMPAIGN_MONITOR_SCHEMA_VERSION.into();
        }
        bijux_dna_infra::write_bytes(&path, serde_json::to_vec(&body)?)?;
        assert_eq!(
            submission_jobs_from_manifest(&path)?["one"].scheduler_job_id.as_deref(),
            Some("42")
        );
    }
    Ok(())
}

#[test]
fn monitoring_reads_only_the_legacy_marker_preview() -> anyhow::Result<()> {
    let root = bijux_dna_infra::temp_dir("campaign-marker-")?;
    let path = root.path().join("bundle");
    let marker = b"\"schema_version\": \"bijux.hpc.bundle.mock_envelope.v1\"";
    let mut bytes = vec![b' '; 512];
    bytes.extend_from_slice(marker);
    bijux_dna_infra::write_bytes(&path, &bytes)?;
    assert!(!is_encrypted_bundle_marker(&path));
    bijux_dna_infra::write_bytes(&path, marker)?;
    assert!(is_encrypted_bundle_marker(&path));
    assert!(!is_encrypted_bundle_marker(&root.path().join("absent")));
    assert_eq!(physical_path(root.path(), "relative/output"), root.path().join("relative/output"));
    Ok(())
}
