use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IdxstatsContigV1 {
    pub contig: String,
    pub length: u64,
    pub mapped: u64,
    pub unmapped: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct IdxstatsSummaryV1 {
    pub contigs: Vec<IdxstatsContigV1>,
    pub total_mapped: u64,
    pub total_unmapped: u64,
    pub reference_mismatch: bool,
}

impl IdxstatsSummaryV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self { contigs: Vec::new(), total_mapped: 0, total_unmapped: 0, reference_mismatch: false }
    }
}

/// # Errors
/// Returns an error if the file cannot be read or parsed.
pub fn parse_samtools_idxstats(path: &Path) -> Result<IdxstatsSummaryV1> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read samtools idxstats: {}", path.display()))?;
    let mut summary = IdxstatsSummaryV1::empty();
    for (line_no, line) in raw.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 4 {
            anyhow::bail!("invalid idxstats line {}: {}", line_no + 1, line);
        }
        let contig = parts[0].to_string();
        let length: u64 = parts[1].parse().unwrap_or(0);
        let mapped: u64 = parts[2].parse().unwrap_or(0);
        let unmapped: u64 = parts[3].parse().unwrap_or(0);
        if contig != "*" && length == 0 && (mapped > 0 || unmapped > 0) {
            summary.reference_mismatch = true;
        }
        if contig == "*" && (mapped > 0 || unmapped > 0) {
            summary.reference_mismatch = true;
        }
        summary.total_mapped = summary.total_mapped.saturating_add(mapped);
        summary.total_unmapped = summary.total_unmapped.saturating_add(unmapped);
        summary.contigs.push(IdxstatsContigV1 { contig, length, mapped, unmapped });
    }
    if summary.contigs.is_empty() {
        anyhow::bail!("samtools idxstats report contains no contig rows");
    }
    Ok(summary)
}
