use anyhow::Context;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ComplexityMetricsV1 {
    pub observed_reads: u64,
    pub projected_reads: Vec<(u64, u64)>,
    #[serde(default)]
    pub saturation_estimate: f64,
}

impl ComplexityMetricsV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            observed_reads: 0,
            projected_reads: Vec::new(),
            saturation_estimate: 0.0,
        }
    }
}

#[allow(clippy::cast_precision_loss)]
fn u64_to_f64(value: u64) -> f64 {
    value as f64
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
    let observed = points.first().map_or(0, |(_, y)| *y);
    let saturation = if points.len() >= 2 {
        let (x0, y0) = points.first().copied().unwrap_or((0, 0));
        let (x1, y1) = points.last().copied().unwrap_or((0, 0));
        if x1 > x0 && y1 > 0 {
            let gain = u64_to_f64(y1.saturating_sub(y0)) / u64_to_f64(x1 - x0);
            (1.0 - gain).clamp(0.0, 1.0)
        } else {
            0.0
        }
    } else {
        0.0
    };
    Ok(ComplexityMetricsV1 {
        observed_reads: observed,
        projected_reads: points,
        saturation_estimate: saturation,
    })
}
