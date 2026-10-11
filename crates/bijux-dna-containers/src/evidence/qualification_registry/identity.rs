//! Owner: bijux-dna-containers
//! Qualification registry namespace and identifier invariants.

use anyhow::{bail, Context, Result};

pub(super) const EXPECTED_REDISTRIBUTABLE_TOOLS: usize = 85;

pub(super) const EXPECTED_EXTERNAL_ARTIFACT_TOOLS: usize = 4;

pub(super) const RUN_NAMESPACE_MINIMUM: u64 = 1000;

pub(super) const RUN_NAMESPACE_MAXIMUM: u64 = 1999;

pub(super) const RUN_NAMESPACE_NEXT: u64 = 1159;

pub(super) const RUN_NAMESPACE_SIZE: usize = 159;

pub(super) fn parse_run_id(value: &str) -> Result<u64> {
    let number = value
        .strip_prefix("run-")
        .with_context(|| format!("qualification run ID is malformed: {value}"))?;
    if number.is_empty() || !number.bytes().all(|byte| byte.is_ascii_digit()) {
        bail!("qualification run ID is malformed: {value}");
    }
    number.parse().context("parse qualification run ID")
}

pub(super) fn parse_run_directory(value: &str) -> Result<Option<(u64, &str)>> {
    let Some(rest) = value.strip_prefix("run-") else {
        return Ok(None);
    };
    let (number, tool) = rest
        .split_once('-')
        .with_context(|| format!("malformed qualification run directory {value}"))?;
    if tool.is_empty() {
        bail!("malformed qualification run directory {value}");
    }
    Ok(Some((number.parse().context("parse qualification run number")?, tool)))
}

pub(super) fn validate_sha256(value: &str, label: &str) -> Result<()> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("{label} SHA-256 is malformed");
    }
    Ok(())
}
