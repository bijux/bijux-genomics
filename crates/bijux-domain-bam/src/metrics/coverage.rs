use anyhow::Context;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CoverageMetricsV1 {
    pub mean: f64,
    pub median: f64,
    pub breadth_1x: f64,
    pub breadth_3x: f64,
    pub breadth_5x: f64,
}

impl CoverageMetricsV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            mean: 0.0,
            median: 0.0,
            breadth_1x: 0.0,
            breadth_3x: 0.0,
            breadth_5x: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CoverageUniformityV1 {
    pub coefficient_of_variation: f64,
    pub dropout_fraction: f64,
}

impl CoverageUniformityV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            coefficient_of_variation: 0.0,
            dropout_fraction: 0.0,
        }
    }
}

impl Default for CoverageUniformityV1 {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EffectiveCoverageV1 {
    pub raw: f64,
    pub dedup: f64,
    pub pmd_filtered: f64,
}

impl EffectiveCoverageV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            raw: 0.0,
            dedup: 0.0,
            pmd_filtered: 0.0,
        }
    }
}

impl Default for EffectiveCoverageV1 {
    fn default() -> Self {
        Self::empty()
    }
}

#[allow(clippy::cast_precision_loss)]
fn u64_to_f64(value: u64) -> f64 {
    value as f64
}

/// # Errors
/// Returns an error if the mosdepth summary cannot be read.
pub fn parse_mosdepth_summary(path: &std::path::Path) -> anyhow::Result<CoverageMetricsV1> {
    let raw = std::fs::read_to_string(path).context("read mosdepth summary")?;
    let mut mean = 0.0;
    let mut breadth_1x = 0.0;
    for line in raw.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }
        if parts[0] == "total" || parts[0] == "genome" || parts[0] == "all" {
            let length = parts[1].parse::<f64>().unwrap_or(0.0);
            let bases_covered = parts[2].parse::<f64>().unwrap_or(0.0);
            mean = parts[3].parse::<f64>().unwrap_or(0.0);
            if length > 0.0 {
                breadth_1x = (bases_covered / length).clamp(0.0, 1.0);
            }
            break;
        }
    }
    Ok(CoverageMetricsV1 {
        mean,
        median: mean,
        breadth_1x,
        breadth_3x: 0.0,
        breadth_5x: 0.0,
    })
}

/// # Errors
/// Returns an error if the samtools depth output cannot be read or parsed.
pub fn parse_samtools_depth(path: &std::path::Path) -> anyhow::Result<CoverageMetricsV1> {
    let raw = std::fs::read_to_string(path).context("read samtools depth")?;
    let mut total_positions = 0_u64;
    let mut total_depth = 0_u64;
    let mut breadth_1x = 0_u64;
    let mut breadth_3x = 0_u64;
    let mut breadth_5x = 0_u64;
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }
        let depth = parts[2].parse::<u64>().unwrap_or(0);
        total_positions += 1;
        total_depth += depth;
        if depth >= 1 {
            breadth_1x += 1;
        }
        if depth >= 3 {
            breadth_3x += 1;
        }
        if depth >= 5 {
            breadth_5x += 1;
        }
    }
    let mean = if total_positions == 0 {
        0.0
    } else {
        u64_to_f64(total_depth) / u64_to_f64(total_positions)
    };
    Ok(CoverageMetricsV1 {
        mean,
        median: mean,
        breadth_1x: if total_positions == 0 {
            0.0
        } else {
            (u64_to_f64(breadth_1x) / u64_to_f64(total_positions)).clamp(0.0, 1.0)
        },
        breadth_3x: if total_positions == 0 {
            0.0
        } else {
            (u64_to_f64(breadth_3x) / u64_to_f64(total_positions)).clamp(0.0, 1.0)
        },
        breadth_5x: if total_positions == 0 {
            0.0
        } else {
            (u64_to_f64(breadth_5x) / u64_to_f64(total_positions)).clamp(0.0, 1.0)
        },
    })
}
