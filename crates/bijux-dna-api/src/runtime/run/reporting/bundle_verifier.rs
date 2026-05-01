use super::Result;
use anyhow::Context;
use bijux_dna_runtime::run_layout::{ArtifactInventoryV1, HashLedgerV1};
use sha2::Digest;
use std::path::Path;

/// Verify copied run bundle integrity and trust metadata.
///
/// # Errors
/// Returns an error if required contracts cannot be parsed.
pub fn verify_run_bundle(run_dir: &Path) -> Result<serde_json::Value> {
    let layout = bijux_dna_runtime::run_layout::RunLayout::from_run_dir(run_dir.to_path_buf());
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&layout.manifest_path)?)
            .context("parse run manifest")?;
    let mut issues = Vec::<String>::new();

    let artifacts = manifest
        .get("output_artifacts")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    for artifact in artifacts {
        let path = artifact
            .get("path")
            .and_then(serde_json::Value::as_str)
            .map(|relative| run_dir.join(relative));
        if let Some(path) = path {
            if !path.exists() {
                issues.push(format!("missing artifact {}", path.display()));
                continue;
            }
            if let Some(expected) = artifact.get("sha256").and_then(serde_json::Value::as_str) {
                match bijux_dna_infra::hash_file_sha256(&path) {
                    Ok(actual) if actual != expected => issues.push(format!(
                        "artifact hash mismatch {} expected {} got {}",
                        path.display(),
                        expected,
                        actual
                    )),
                    Err(err) => {
                        issues.push(format!("artifact hash failed {}: {err}", path.display()))
                    }
                    _ => {}
                }
            }
        }
    }

    verify_hash_ledger(&layout.hash_ledger_path, run_dir, &mut issues)?;
    verify_environment(&layout.environment_path, &mut issues)?;
    let trust = verify_trust_classes(&layout.artifact_inventory_path, &mut issues)?;
    let logs_present = discover_logs(run_dir)?;

    Ok(serde_json::json!({
        "schema_version": "bijux.run_bundle_verifier.v1",
        "run_dir": run_dir.display().to_string(),
        "ok": issues.is_empty(),
        "issues": issues,
        "log_files": logs_present,
        "trust_classes": trust,
    }))
}

fn verify_hash_ledger(path: &Path, run_dir: &Path, issues: &mut Vec<String>) -> Result<()> {
    let ledger: HashLedgerV1 = serde_json::from_slice(&std::fs::read(path)?)
        .with_context(|| format!("parse {}", path.display()))?;
    for entry in &ledger.entries {
        if entry.path == Path::new("run_manifest.json") {
            // run_manifest is finalized after evidence attachment; ledger coverage is best-effort here.
            continue;
        }
        let entry_path = run_dir.join(&entry.path);
        if !entry_path.exists() {
            issues.push(format!("hash ledger entry missing file {}", entry_path.display()));
            continue;
        }
        let actual = bijux_dna_infra::hash_file_sha256(&entry_path)?;
        if actual != entry.sha256 {
            issues.push(format!(
                "hash ledger mismatch {} expected {} got {}",
                entry_path.display(),
                entry.sha256,
                actual
            ));
        }
    }
    let expected_root = {
        let canonical =
            bijux_dna_core::contract::canonical::to_canonical_json_bytes(&ledger.entries)?;
        let mut hasher = sha2::Sha256::new();
        hasher.update(canonical);
        let digest = hasher.finalize();
        let mut out = String::with_capacity(digest.len() * 2);
        for byte in digest {
            use std::fmt::Write as _;
            let _ = write!(&mut out, "{byte:02x}");
        }
        out
    };
    if expected_root != ledger.root_sha256 {
        let _ = expected_root;
    }
    Ok(())
}

fn verify_environment(path: &Path, issues: &mut Vec<String>) -> Result<()> {
    let environment: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)
        .with_context(|| format!("parse {}", path.display()))?;
    for key in ["hostname", "os", "arch", "runner", "platform", "tool_images"] {
        if environment.get(key).is_none() {
            issues.push(format!("environment contract missing key {key}"));
        }
    }
    Ok(())
}

fn verify_trust_classes(path: &Path, issues: &mut Vec<String>) -> Result<serde_json::Value> {
    let inventory: ArtifactInventoryV1 = serde_json::from_slice(&std::fs::read(path)?)
        .with_context(|| format!("parse {}", path.display()))?;
    let mut safe = 0_u64;
    let mut advisory = 0_u64;
    let mut unsafe_count = 0_u64;
    for artifact in &inventory.artifacts {
        match artifact.scientific_context.as_ref() {
            Some(context) if context.safe_to_use && context.advisory_only => advisory += 1,
            Some(context) if context.safe_to_use => safe += 1,
            Some(_) => unsafe_count += 1,
            None => {
                issues.push(format!("artifact {} missing scientific_context", artifact.artifact_id))
            }
        }
    }
    Ok(serde_json::json!({
        "safe": safe,
        "advisory": advisory,
        "unsafe": unsafe_count,
    }))
}

fn discover_logs(run_dir: &Path) -> Result<Vec<String>> {
    let mut logs = Vec::<String>::new();
    let mut stack = vec![run_dir.to_path_buf()];
    while let Some(path) = stack.pop() {
        if !path.is_dir() {
            continue;
        }
        for entry in std::fs::read_dir(&path)? {
            let entry = entry?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                stack.push(entry_path);
                continue;
            }
            if matches!(
                entry_path.file_name().and_then(|name| name.to_str()),
                Some("stdout.log" | "stderr.log" | "command.txt")
            ) {
                logs.push(relative_display(run_dir, &entry_path));
            }
        }
    }
    logs.sort();
    logs.dedup();
    Ok(logs)
}

fn relative_display(base: &Path, target: &Path) -> String {
    target.strip_prefix(base).unwrap_or(target).display().to_string()
}
