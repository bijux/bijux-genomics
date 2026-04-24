use anyhow::{anyhow, Result};
use serde_json::Value;
use sha2::Digest;
use std::fmt::Write as _;

use bijux_dna_core::prelude::hashing::input_fingerprint;

pub(super) fn canonical_sha256(value: &Value) -> Result<String> {
    let bytes = bijux_dna_core::contract::canonical::to_canonical_json_bytes(value)?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    Ok(sha256_hex(hasher.finalize()))
}

pub(super) fn declared_json_array<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("run manifest missing declared `{key}` array"))
}

pub(super) fn manifest_sort_key(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(|entry| entry.as_str())
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .unwrap_or("not_declared")
        .to_string()
}

pub(super) fn detect_run_context() -> Result<crate::RunContextV1> {
    let mode = std::env::var("BIJUX_RUN_CONTEXT").unwrap_or_else(|_| "local".to_string());
    if mode.eq_ignore_ascii_case("hpc") {
        let site = std::env::var("BIJUX_HPC_SITE")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| anyhow!("HPC run context requires BIJUX_HPC_SITE"))?;
        let scratch = std::env::var("TMPDIR")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| anyhow!("HPC run context requires TMPDIR"))?;
        let slurm = std::env::var("SLURM_JOB_ID").is_ok();
        Ok(crate::RunContextV1::Hpc { site, scratch, slurm })
    } else {
        Ok(crate::RunContextV1::Local)
    }
}

#[must_use]
pub fn compute_run_id(
    stage: &str,
    tool: &str,
    image_digest: &str,
    input_hash: &str,
    params_hash: &str,
) -> String {
    let seed = format!("{stage}|{tool}|{image_digest}|{input_hash}|{params_hash}");
    let mut hasher = sha2::Sha256::new();
    hasher.update(seed.as_bytes());
    sha256_hex(hasher.finalize())
}

#[must_use]
pub(super) fn input_hash_from_many(values: &[String]) -> String {
    input_fingerprint(values)
}

fn sha256_hex(digest: impl AsRef<[u8]>) -> String {
    let bytes = digest.as_ref();
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut hex, "{byte:02x}");
    }
    hex
}
