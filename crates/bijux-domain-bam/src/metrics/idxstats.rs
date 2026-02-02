use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;

use anyhow::{bail, Context, Result};

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
        Self {
            contigs: Vec::new(),
            total_mapped: 0,
            total_unmapped: 0,
            reference_mismatch: false,
        }
    }
}

/// Parse a samtools idxstats output file into a canonical summary.
///
/// # Errors
/// Returns an error if the file cannot be read or is malformed.
pub fn parse_samtools_idxstats(path: &Path) -> Result<IdxstatsSummaryV1> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("read samtools idxstats from {}", path.display()))?;
    let mut contigs = Vec::new();
    let mut total_mapped = 0u64;
    let mut total_unmapped = 0u64;
    let mut reference_mismatch = false;

    for (idx, line) in raw.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() != 4 {
            bail!("idxstats line {} has {} fields", idx + 1, parts.len());
        }
        let contig = parts[0].to_string();
        let length: u64 = parts[1].parse().with_context(|| {
            format!("idxstats line {} invalid length {}", idx + 1, parts[1])
        })?;
        let mapped: u64 = parts[2].parse().with_context(|| {
            format!("idxstats line {} invalid mapped {}", idx + 1, parts[2])
        })?;
        let unmapped: u64 = parts[3].parse().with_context(|| {
            format!(
                "idxstats line {} invalid unmapped {}",
                idx + 1,
                parts[3]
            )
        })?;

        if contig == "*" && (mapped > 0 || unmapped > 0) {
            reference_mismatch = true;
        }
        if contig != "*" && length == 0 {
            reference_mismatch = true;
        }

        total_mapped = total_mapped.saturating_add(mapped);
        total_unmapped = total_unmapped.saturating_add(unmapped);
        contigs.push(IdxstatsContigV1 {
            contig,
            length,
            mapped,
            unmapped,
        });
    }

    Ok(IdxstatsSummaryV1 {
        contigs,
        total_mapped,
        total_unmapped,
        reference_mismatch,
    })
}
