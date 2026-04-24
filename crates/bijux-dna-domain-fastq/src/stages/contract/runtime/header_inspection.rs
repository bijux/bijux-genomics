use std::path::Path;

use anyhow::{anyhow, Result};
use tracing::warn;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HeaderInspection {
    pub warnings: Vec<String>,
}

/// Inspect FASTQ headers for pairing and style drift.
///
/// # Errors
/// Returns an error in strict mode if mismatches are detected.
pub fn inspect_headers(r1: &Path, r2: Option<&Path>, strict: bool) -> Result<HeaderInspection> {
    let mut warnings = Vec::new();
    let r1_names = read_header_names(r1, 16)?;
    if let Some(r2) = r2 {
        let r2_names = read_header_names(r2, 16)?;
        for (idx, (left, right)) in r1_names.iter().zip(r2_names.iter()).enumerate() {
            if normalize_header(left) != normalize_header(right) {
                let msg = format!("pairing mismatch at record {idx}: {left} vs {right}");
                if strict {
                    return Err(anyhow!(msg));
                }
                warnings.push(msg);
                break;
            }
        }
    }
    if has_style_drift(&r1_names) {
        let msg = "header style drift detected".to_string();
        if strict {
            return Err(anyhow!(msg));
        }
        warnings.push(msg);
    }
    Ok(HeaderInspection { warnings })
}

pub fn log_header_warnings(stage_id: &str, inspection: &HeaderInspection) {
    for warning in &inspection.warnings {
        warn!(stage = stage_id, "{warning}");
    }
}

/// Ensure UMI headers are present before UMI stage execution.
///
/// # Errors
/// Returns an error if UMI markers are not detected and override is not set.
pub fn ensure_umi_headers(r1: &Path, r2: Option<&Path>) -> Result<()> {
    let mut names = read_header_names(r1, 32)?;
    if let Some(r2) = r2 {
        names.extend(read_header_names(r2, 32)?);
    }
    let markers = ["UMI", "RX:", "BX:", "UB:"];
    let has_marker = names.iter().any(|name| markers.iter().any(|marker| name.contains(marker)));
    if has_marker {
        return Ok(());
    }
    if std::env::var("BIJUX_ALLOW_NO_UMI").is_ok() {
        warn!("UMI headers not detected; proceeding due to BIJUX_ALLOW_NO_UMI");
        return Ok(());
    }
    Err(anyhow!("UMI headers not detected; set BIJUX_ALLOW_NO_UMI=1 to bypass"))
}

fn read_header_names(path: &Path, max_records: usize) -> Result<Vec<String>> {
    let data = super::read_fastq_text(path)?;
    let mut names = Vec::new();
    for (idx, line) in data.lines().enumerate() {
        if idx % 4 == 0 {
            if let Some(name) = line.strip_prefix('@') {
                names.push(name.trim().to_string());
                if names.len() >= max_records {
                    break;
                }
            }
        }
    }
    Ok(names)
}

fn normalize_header(name: &str) -> String {
    let name = name.split_whitespace().next().unwrap_or(name);
    name.trim_end_matches("/1").trim_end_matches("/2").to_string()
}

fn has_style_drift(names: &[String]) -> bool {
    if names.is_empty() {
        return false;
    }
    let first = normalize_header(&names[0]);
    names.iter().any(|name| normalize_header(name) != first)
}
