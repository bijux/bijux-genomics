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
        Self { mean: 0.0, median: 0.0, breadth_1x: 0.0, breadth_3x: 0.0, breadth_5x: 0.0 }
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
        Self { coefficient_of_variation: 0.0, dropout_fraction: 0.0 }
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
        Self { raw: 0.0, dedup: 0.0, pmd_filtered: 0.0 }
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
    let mut found_summary_row = false;
    for (line_no, line) in raw.lines().enumerate() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }
        if parts[0] == "total" || parts[0] == "genome" || parts[0] == "all" {
            let length = parts[1]
                .parse::<f64>()
                .with_context(|| format!("parse mosdepth length on line {}", line_no + 1))?;
            let bases_covered = parts[2]
                .parse::<f64>()
                .with_context(|| format!("parse mosdepth covered bases on line {}", line_no + 1))?;
            mean = parts[3]
                .parse::<f64>()
                .with_context(|| format!("parse mosdepth mean depth on line {}", line_no + 1))?;
            if length > 0.0 {
                breadth_1x = (bases_covered / length).clamp(0.0, 1.0);
            }
            found_summary_row = true;
            break;
        }
    }
    if !found_summary_row {
        anyhow::bail!("mosdepth summary missing total/genome/all coverage row");
    }
    Ok(CoverageMetricsV1 { mean, median: mean, breadth_1x, breadth_3x: 0.0, breadth_5x: 0.0 })
}

/// # Errors
/// Returns an error if the samtools depth output cannot be read or parsed.
pub fn parse_samtools_depth(path: &std::path::Path) -> anyhow::Result<CoverageMetricsV1> {
    Ok(parse_samtools_depth_with_uniformity(path)?.0)
}

/// # Errors
/// Returns an error if the samtools depth output cannot be read or parsed.
pub fn parse_samtools_depth_with_uniformity(
    path: &std::path::Path,
) -> anyhow::Result<(CoverageMetricsV1, CoverageUniformityV1)> {
    let raw = std::fs::read_to_string(path).context("read samtools depth")?;
    let mut total_positions = 0_u64;
    let mut total_depth = 0_u64;
    let mut breadth_1x = 0_u64;
    let mut breadth_3x = 0_u64;
    let mut breadth_5x = 0_u64;
    let mut sum_sq = 0_f64;
    for (line_no, line) in raw.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            anyhow::bail!("samtools depth line {} has {} columns", line_no + 1, parts.len());
        }
        let depth = parts[2]
            .parse::<u64>()
            .with_context(|| format!("parse samtools depth on line {}", line_no + 1))?;
        total_positions += 1;
        total_depth += depth;
        sum_sq += u64_to_f64(depth).powi(2);
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
    if total_positions == 0 {
        anyhow::bail!("samtools depth report contains no coverage rows");
    }
    let mean = if total_positions == 0 {
        0.0
    } else {
        u64_to_f64(total_depth) / u64_to_f64(total_positions)
    };
    let variance = if total_positions == 0 {
        0.0
    } else {
        (sum_sq / u64_to_f64(total_positions)) - mean.powi(2)
    };
    let stddev = variance.max(0.0).sqrt();
    let cv = if mean > 0.0 { stddev / mean } else { 0.0 };
    let dropout_fraction = if total_positions == 0 {
        0.0
    } else {
        1.0 - (u64_to_f64(breadth_1x) / u64_to_f64(total_positions)).clamp(0.0, 1.0)
    };
    let coverage = CoverageMetricsV1 {
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
    };
    let uniformity = CoverageUniformityV1 { coefficient_of_variation: cv, dropout_fraction };
    Ok((coverage, uniformity))
}
