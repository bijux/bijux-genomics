use anyhow::{anyhow, Result};
use bijux_dna_api::v1::api::run::{execute_local_fastq_workflow, verify_run_bundle};
use std::path::Path;

#[test]
fn run_bundle_verifier_validates_original_and_copied_bundles() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let response = execute_local_fastq_workflow(&temp.path().join("source"))?;
    let run_dir =
        response.manifest_path.parent().ok_or_else(|| anyhow!("manifest missing parent"))?;
    let original = verify_run_bundle(run_dir)?;
    if original.get("ok").and_then(serde_json::Value::as_bool) != Some(true) {
        panic!("original bundle verification failed: {}", original);
    }
    assert!(original.get("log_files").is_some());

    let copied_dir = temp.path().join("copied-run");
    copy_tree(run_dir, &copied_dir)?;
    let copied = verify_run_bundle(&copied_dir)?;
    if copied.get("ok").and_then(serde_json::Value::as_bool) != Some(true) {
        panic!("copied bundle verification failed: {}", copied);
    }
    assert!(copied.get("trust_classes").is_some());
    Ok(())
}

fn copy_tree(src: &Path, dst: &Path) -> Result<()> {
    if !src.is_dir() {
        return Err(anyhow!("missing source directory {}", src.display()));
    }
    bijux_dna_infra::ensure_dir(dst)?;
    let mut stack = vec![(src.to_path_buf(), dst.to_path_buf())];
    while let Some((current_src, current_dst)) = stack.pop() {
        for entry in std::fs::read_dir(&current_src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = current_dst.join(entry.file_name());
            if src_path.is_dir() {
                bijux_dna_infra::ensure_dir(&dst_path)?;
                stack.push((src_path, dst_path));
                continue;
            }
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

#[test]
fn run_bundle_verifier_rejects_changed_root_with_matching_artifact_digest() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let response = execute_local_fastq_workflow(&temp.path().join("source"))?;
    let run_dir =
        response.manifest_path.parent().ok_or_else(|| anyhow!("manifest missing parent"))?;
    let source_ledger_path = run_dir.join("hash_ledger.json");
    let source_ledger_bytes = std::fs::read(&source_ledger_path)?;
    let original = verify_run_bundle(run_dir)?;
    assert_eq!(original.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let copied_dir = temp.path().join("copied-run");
    copy_tree(run_dir, &copied_dir)?;
    let copied_ledger_path = copied_dir.join("hash_ledger.json");
    let mut ledger: serde_json::Value = serde_json::from_slice(&source_ledger_bytes)?;
    let original_entries = ledger["entries"].clone();
    let original_root = ledger["root_sha256"]
        .as_str()
        .ok_or_else(|| anyhow!("producer ledger root is missing"))?
        .to_owned();
    assert_eq!(original_root.len(), 64);
    assert!(original_root.bytes().all(|byte| byte.is_ascii_hexdigit()));
    let replacement = if original_root.starts_with('0') { '1' } else { '0' };
    let changed_root = format!("{replacement}{}", &original_root[1..]);
    ledger["root_sha256"] = serde_json::Value::String(changed_root.clone());
    std::fs::write(&copied_ledger_path, serde_json::to_vec(&ledger)?)?;

    let copied_manifest_path = copied_dir.join("run_manifest.json");
    let mut copied_manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&copied_manifest_path)?)?;
    let declared_ledger = copied_manifest["output_artifacts"]
        .as_array_mut()
        .and_then(|artifacts| {
            artifacts.iter_mut().find(|artifact| {
                artifact.get("path").and_then(serde_json::Value::as_str) == Some("hash_ledger.json")
            })
        })
        .ok_or_else(|| anyhow!("producer did not declare the hash-ledger artifact"))?;
    declared_ledger["sha256"] =
        serde_json::Value::String(bijux_dna_infra::hash_file_sha256(&copied_ledger_path)?);
    std::fs::write(&copied_manifest_path, serde_json::to_vec(&copied_manifest)?)?;
    assert_eq!(ledger["entries"], original_entries);
    assert_eq!(std::fs::read(&source_ledger_path)?, source_ledger_bytes);

    let result = verify_run_bundle(&copied_dir)?;
    eprintln!("ledger root corruption with matching artifact digest: {result}");
    assert_eq!(result.get("ok").and_then(serde_json::Value::as_bool), Some(false));
    let issues = result["issues"].as_array().ok_or_else(|| anyhow!("issues missing"))?;
    assert!(issues.iter().filter_map(serde_json::Value::as_str).any(|issue| {
        issue.contains("hash ledger root mismatch")
            && issue.contains(&original_root)
            && issue.contains(&changed_root)
    }));
    assert_eq!(ledger["entries"], original_entries);
    assert_eq!(std::fs::read(&source_ledger_path)?, source_ledger_bytes);
    assert_eq!(
        verify_run_bundle(run_dir)?.get("ok").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    Ok(())
}
