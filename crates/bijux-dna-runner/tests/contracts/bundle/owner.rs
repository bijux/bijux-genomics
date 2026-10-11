#![allow(clippy::expect_used)]

use super::{
    decrypt_bundle, sha256_hex, sidecar_path_for, write_encrypted_bundle, BundleDecryptRequest,
    BundleWriteRequest,
};

#[test]
fn mock_bundle_round_trip_and_sidecar_hash_validation() {
    let root = tempfile::tempdir().expect("tempdir");
    let bundle_path = root.path().join("bundle.results");
    let plaintext = br#"{"metrics":{"walltime_sec":10}}"#;
    let recipients = vec!["alice".to_string(), "bob".to_string()];

    let sidecar = write_encrypted_bundle(&BundleWriteRequest {
        output_path: &bundle_path,
        bundle_kind: "results",
        campaign_id: "mini",
        domain: "fastq",
        stage: "fastq.validate_reads",
        tool: "seqkit_v2",
        sample: "sample-1",
        planned_job_id: "dryrun-fastq-validate-reads",
        scheduler_job_id: "mock-0001",
        submitted_at: "1700000000",
        backend: "mock-envelope-v1",
        recipients: &recipients,
        plaintext,
    })
    .expect("encrypt bundle");
    assert_eq!(sidecar.plaintext_sha256, sha256_hex(plaintext));
    assert!(bundle_path.is_file());

    let (decoded_sidecar, decoded) = decrypt_bundle(&BundleDecryptRequest {
        bundle_path: &bundle_path,
        sidecar_path: None,
        identity_files: &[],
    })
    .expect("decrypt bundle");
    assert_eq!(decoded_sidecar.bundle_kind, "results");
    assert_eq!(decoded, plaintext);
}

#[test]
fn decrypt_bundle_detects_ciphertext_tamper() {
    let root = tempfile::tempdir().expect("tempdir");
    let bundle_path = root.path().join("bundle.results");
    let plaintext = br#"{"metrics":{"walltime_sec":10}}"#;
    let recipients = vec!["alice".to_string()];

    write_encrypted_bundle(&BundleWriteRequest {
        output_path: &bundle_path,
        bundle_kind: "results",
        campaign_id: "mini",
        domain: "fastq",
        stage: "fastq.validate_reads",
        tool: "seqkit_v2",
        sample: "sample-1",
        planned_job_id: "dryrun-fastq-validate-reads",
        scheduler_job_id: "mock-0001",
        submitted_at: "1700000000",
        backend: "mock-envelope-v1",
        recipients: &recipients,
        plaintext,
    })
    .expect("encrypt bundle");

    let mut ciphertext = std::fs::read(&bundle_path).expect("read ciphertext");
    ciphertext[0] ^= 1;
    bijux_dna_infra::write_bytes(&bundle_path, ciphertext).expect("write tampered ciphertext");

    let err = decrypt_bundle(&BundleDecryptRequest {
        bundle_path: &bundle_path,
        sidecar_path: None,
        identity_files: &[],
    })
    .expect_err("must fail integrity validation");
    assert!(err.to_string().contains("ciphertext hash mismatch"));
}

#[test]
fn age_backend_refuses_decrypt_without_identity_files() {
    let root = tempfile::tempdir().expect("tempdir");
    let bundle_path = root.path().join("bundle.code");
    let plaintext = br#"{"code":{"state":"frozen"}}"#;
    let recipients = vec!["alice".to_string()];

    write_encrypted_bundle(&BundleWriteRequest {
        output_path: &bundle_path,
        bundle_kind: "code",
        campaign_id: "mini",
        domain: "fastq",
        stage: "fastq.validate_reads",
        tool: "seqkit_v2",
        sample: "sample-1",
        planned_job_id: "dryrun-fastq-validate-reads",
        scheduler_job_id: "mock-0001",
        submitted_at: "1700000000",
        backend: "mock-envelope-v1",
        recipients: &recipients,
        plaintext,
    })
    .expect("encrypt bundle");

    let sidecar_path = sidecar_path_for(&bundle_path);
    let mut sidecar: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&sidecar_path).expect("read sidecar"))
            .expect("parse sidecar");
    sidecar["backend"] = serde_json::Value::String("age-cli".to_string());
    bijux_dna_infra::write_bytes(
        &sidecar_path,
        serde_json::to_vec_pretty(&sidecar).expect("serialize sidecar"),
    )
    .expect("write sidecar");

    let err = decrypt_bundle(&BundleDecryptRequest {
        bundle_path: &bundle_path,
        sidecar_path: Some(&sidecar_path),
        identity_files: &[],
    })
    .expect_err("must reject missing age identity");
    assert!(err.to_string().contains("requires at least one identity file"));
}

#[test]
fn bundle_sidecar_rejects_unknown_contracts_and_false_plaintext_counts() -> anyhow::Result<()> {
    let root = tempfile::tempdir()?;
    let path = root.path().join("bundle.results");
    let recipients = vec!["recipient".to_string()];
    let sidecar = write_encrypted_bundle(&BundleWriteRequest {
        output_path: &path,
        bundle_kind: "results",
        campaign_id: "campaign",
        domain: "fastq",
        stage: "fastq.validate_reads",
        tool: "seqkit_v2",
        sample: "sample",
        planned_job_id: "planned",
        scheduler_job_id: "mock-0001",
        submitted_at: "1700000000",
        backend: "mock-envelope-v1",
        recipients: &recipients,
        plaintext: &[255, 0, 128, 10],
    })?;
    let wire = serde_json::to_value(&sidecar)?;
    for (field, value) in [
        ("schema_version", serde_json::json!("unknown.v2")),
        ("backend", serde_json::json!("unsupported")),
        ("plaintext_sha256", serde_json::json!("not-a-digest")),
        ("plaintext_bytes", serde_json::json!(-1)),
        ("unknown_field", serde_json::json!(true)),
    ] {
        let mut invalid = wire.clone();
        invalid[field] = value;
        assert!(serde_json::from_value::<super::BundleSidecar>(invalid).is_err());
    }
    let mut inconsistent = wire;
    inconsistent["plaintext_bytes"] = serde_json::json!(5);
    bijux_dna_infra::write_bytes(sidecar_path_for(&path), serde_json::to_vec(&inconsistent)?)?;
    let error = decrypt_bundle(&BundleDecryptRequest {
        bundle_path: &path,
        sidecar_path: None,
        identity_files: &[],
    })
    .err()
    .ok_or_else(|| anyhow::anyhow!("false plaintext count was admitted"))?;
    assert!(format!("{error:#}").contains("byte count mismatch"));
    Ok(())
}

#[test]
fn plaintext_staging_is_private_during_execution_and_removed_after_success() -> anyhow::Result<()> {
    let root = tempfile::tempdir()?;
    let output = root.path().join("ciphertext");
    let payload = [255, 0, 128, 10];
    let mut observed = None;
    let result = super::storage::with_staged_plaintext(&output, &payload, |path| {
        assert_eq!(path.parent(), Some(root.path()));
        assert_eq!(std::fs::read(path)?, payload);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(std::fs::metadata(path)?.permissions().mode() & 0o777, 0o600);
        }
        observed = Some(path.to_path_buf());
        Ok(42)
    })?;
    assert_eq!(result, 42);
    assert!(!observed.expect("backend observed plaintext").exists());
    assert_eq!(std::fs::read_dir(root.path())?.count(), 0);
    Ok(())
}

#[test]
fn plaintext_staging_retains_backend_and_cleanup_failures() -> anyhow::Result<()> {
    let root = tempfile::tempdir()?;
    let output = root.path().join("ciphertext");
    let error = super::storage::with_staged_plaintext::<()>(&output, b"private", |_path| {
        Err(anyhow::anyhow!("backend refused recipient"))
    })
    .expect_err("backend failure");
    assert!(error.to_string().contains("backend refused recipient"));
    assert_eq!(std::fs::read_dir(root.path())?.count(), 0);
    let error = super::storage::with_staged_plaintext::<()>(&output, b"private", |path| {
        // Replacing only this owned fixture makes close fail independently of the backend.
        bijux_dna_infra::remove_file(path)?;
        bijux_dna_infra::ensure_dir(path)?;
        Err(anyhow::anyhow!("backend refused recipient"))
    })
    .expect_err("both failures must survive");
    let message = format!("{error:#}");
    assert!(message.contains("backend refused recipient"));
    assert!(message.contains("remove private bundle plaintext"));
    Ok(())
}

fn recipient_request<'a>(
    path: &'a std::path::Path,
    recipients: &'a [String],
) -> BundleWriteRequest<'a> {
    BundleWriteRequest {
        output_path: path,
        bundle_kind: "results",
        campaign_id: "recipient-admission",
        domain: "test",
        stage: "test",
        tool: "test",
        sample: "test",
        planned_job_id: "plan",
        scheduler_job_id: "mock",
        submitted_at: "1700000000",
        backend: "mock-envelope-v1",
        recipients,
        plaintext: b"retained payload",
    }
}

#[test]
fn recipient_admission_rejects_invalid_declarations_before_filesystem_effects() -> anyhow::Result<()>
{
    let root = tempfile::tempdir()?;
    let output = root.path().join("not-created/bundle");
    for recipients in [
        vec![],
        vec![String::new()],
        vec!["  ".into()],
        vec![" alice".into()],
        vec!["alice".into(), "alice".into()],
    ] {
        assert!(write_encrypted_bundle(&recipient_request(&output, &recipients)).is_err());
        assert!(!root.path().join("not-created").exists());
    }
    Ok(())
}

#[test]
fn recipient_admission_rejects_sidecar_and_mock_envelope_contradictions() -> anyhow::Result<()> {
    let root = tempfile::tempdir()?;
    let output = root.path().join("bundle");
    let recipients = vec!["alice".into(), "bob".into()];
    let sidecar = write_encrypted_bundle(&recipient_request(&output, &recipients))?;
    let request =
        BundleDecryptRequest { bundle_path: &output, sidecar_path: None, identity_files: &[] };
    let sidecar_path = sidecar_path_for(&output);
    let original = std::fs::read(&sidecar_path)?;
    for replacement in [
        vec![],
        vec!["incorrect".to_string()],
        sidecar.recipient_fingerprints.iter().rev().cloned().collect(),
    ] {
        let mut changed = sidecar.clone();
        changed.recipient_fingerprints = replacement;
        std::fs::write(&sidecar_path, serde_json::to_vec(&changed)?)?;
        let error = decrypt_bundle(&request).expect_err("contradictory fingerprints");
        assert!(format!("{error:#}").contains("fingerprints disagree"));
    }
    let mut changed = sidecar;
    changed.recipients = vec!["mallory".into()];
    changed.recipient_fingerprints = vec![super::storage::recipient_fingerprint("mallory")];
    std::fs::write(&sidecar_path, serde_json::to_vec(&changed)?)?;
    let error = decrypt_bundle(&request).expect_err("mock envelope declares different recipients");
    assert!(format!("{error:#}").contains("envelope recipients disagree"));
    std::fs::write(&sidecar_path, &original)?;
    assert_eq!(decrypt_bundle(&request)?.1, b"retained payload");
    Ok(())
}

#[test]
fn code_freeze_observation_reads_current_script_and_rejects_missing_source() -> anyhow::Result<()> {
    let root = tempfile::tempdir()?;
    let script = root.path().join("job.sbatch");
    std::fs::write(&script, "first script\n")?;
    assert_eq!(super::observe_hpc_code_freeze(&script)?.slurm_script, "first script\n");
    std::fs::write(&script, "replacement script\n")?;
    assert_eq!(super::observe_hpc_code_freeze(&script)?.slurm_script, "replacement script\n");
    std::fs::remove_file(&script)?;
    assert!(super::observe_hpc_code_freeze(&script).is_err());
    Ok(())
}
