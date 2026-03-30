use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use serde_json::Value;
use sha2::{Digest, Sha256};

fn repo_root() -> Result<PathBuf> {
    crate::support::repo_root().map_err(|err| anyhow!("workspace root not found: {err}"))
}

fn golden_fastq_toy_dir() -> Result<PathBuf> {
    Ok(repo_root()?.join("assets/golden/toy-runs-v1/fastq_reference_adna"))
}

#[test]
fn fastq_small_pipeline_emits_multi_stage_manifest() -> Result<()> {
    let run_dir = golden_fastq_toy_dir()?;
    let manifest_path = run_dir.join("manifest.json");
    let checksums_path = run_dir.join("artifact_checksums.json");
    let metrics_path = run_dir.join("metrics.json");
    let raw = std::fs::read_to_string(&manifest_path)?;
    let manifest: serde_json::Value = serde_json::from_str(&raw)?;
    assert_eq!(
        manifest
            .get("profile_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| panic!(
                "manifest missing string profile_id: {}",
                manifest_path.display()
            )),
        "fastq_reference_adna"
    );
    let checksums: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&checksums_path)?)?;
    let artifacts = checksums
        .get("artifacts")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| anyhow!("artifact_checksums missing artifacts object"))?;
    assert!(
        (3..=5).contains(&artifacts.len()),
        "expected 3-5 stable artifacts, got {}",
        artifacts.len()
    );
    let metrics: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&metrics_path)?)?;
    for key in ["reads_total", "bases_total", "retention_ratio"] {
        assert!(
            metrics.get(key).is_some(),
            "metrics.json missing required key `{key}`"
        );
    }
    Ok(())
}

#[test]
fn fastq_small_golden_checksums_match_materialized_files() -> Result<()> {
    let run_dir = golden_fastq_toy_dir()?;
    let checksums: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
        run_dir.join("artifact_checksums.json"),
    )?)?;
    let artifacts = checksums
        .get("artifacts")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| anyhow!("artifact_checksums missing artifacts object"))?;

    for (name, expected) in artifacts {
        let expected = expected
            .as_str()
            .ok_or_else(|| anyhow!("artifact checksum for `{name}` must be a string"))?;
        let actual = stable_toy_digest(&run_dir.join(name))?;
        assert_eq!(
            actual, expected,
            "golden toy checksum drift detected for `{name}`"
        );
    }
    Ok(())
}

fn stable_toy_digest(path: &Path) -> Result<String> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("json") => {
            let payload = normalize_toy_json(&read_json_value(path)?);
            Ok(sha256_hex_bytes(
                serde_json::to_string(&payload)?.as_bytes(),
            ))
        }
        Some("html") => Ok(sha256_hex_bytes(
            normalize_toy_html(&std::fs::read_to_string(path)?)?.as_bytes(),
        )),
        _ => {
            let bytes = std::fs::read(path)?;
            Ok(sha256_hex_bytes(&bytes))
        }
    }
}

fn read_json_value(path: &Path) -> Result<Value> {
    serde_json::from_str(&std::fs::read_to_string(path)?)
        .with_context(|| format!("parse {}", path.display()))
}

fn normalize_toy_json(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .filter(|(key, _)| {
                    !matches!(
                        key.as_str(),
                        "generated_at" | "timestamp" | "started_at" | "finished_at"
                    )
                })
                .map(|(key, value)| (key.clone(), normalize_toy_json(value)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(normalize_toy_json).collect()),
        _ => value.clone(),
    }
}

fn normalize_toy_html(raw: &str) -> Result<String> {
    let generated_re =
        Regex::new(r"generated_at=[^<]+").context("compile html generated_at regex")?;
    let json_re =
        Regex::new(r#""generated_at"\s*:\s*"[^"]+""#).context("compile json generated_at regex")?;
    let text = generated_re.replace_all(raw, "generated_at=<normalized>");
    Ok(json_re
        .replace_all(&text, r#""generated_at":"<normalized>""#)
        .into_owned())
}

fn sha256_hex_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
