use anyhow::Context;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ComplexityMetricsV1 {
    pub observed_reads: u64,
    pub projected_reads: Vec<(u64, u64)>,
}

impl ComplexityMetricsV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            observed_reads: 0,
            projected_reads: Vec::new(),
        }
    }
}

/// # Errors
/// Returns an error if the preseq output cannot be read.
pub fn parse_preseq_estimates(path: &std::path::Path) -> anyhow::Result<ComplexityMetricsV1> {
    let raw = std::fs::read_to_string(path).context("read preseq output")?;
    let mut points = Vec::new();
    for line in raw.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let (Ok(x), Ok(y)) = (parts[0].parse::<u64>(), parts[1].parse::<u64>()) {
                points.push((x, y));
            }
        }
    }
    Ok(ComplexityMetricsV1 {
        observed_reads: points.first().map_or(0, |(_, y)| *y),
        projected_reads: points,
    })
}
