//! Owner: bijux-dna-bench
//! Scheduler resource observations and metric distributions for stage reports.

use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::BenchmarkMetricDistribution;

#[derive(Debug, Clone, Copy)]
pub(in crate::workflow) struct ResourceObservation {
    pub(in crate::workflow) wall_seconds: f64,
    pub(in crate::workflow) user_seconds: f64,
    pub(in crate::workflow) system_seconds: f64,
    pub(in crate::workflow) peak_rss_kib: f64,
    pub(in crate::workflow) filesystem_input_operations: f64,
    pub(in crate::workflow) filesystem_output_operations: f64,
}

pub(in crate::workflow) fn read_resource_observation(path: &Path) -> Result<ResourceObservation> {
    let bytes = bijux_dna_infra::read_to_end_bounded(path, 64 * 1024)
        .with_context(|| format!("read benchmark resources {}", path.display()))?;
    let text = std::str::from_utf8(&bytes).context("benchmark resource TSV is not UTF-8")?;
    let lines = text.lines().collect::<Vec<_>>();
    if lines.len() != 2
        || lines[0]
            != "wall_seconds\tuser_seconds\tsystem_seconds\tmax_rss_kb\tfilesystem_inputs\tfilesystem_outputs\texit_code"
    {
        bail!("benchmark resource TSV shape is invalid");
    }
    let fields = lines[1].split('\t').collect::<Vec<_>>();
    if fields.len() != 7 || fields[6] != "0" {
        bail!("benchmark resource TSV records an invalid execution");
    }
    let observation = ResourceObservation {
        wall_seconds: parse_nonnegative(fields[0], "wall seconds")?,
        user_seconds: parse_nonnegative(fields[1], "user seconds")?,
        system_seconds: parse_nonnegative(fields[2], "system seconds")?,
        peak_rss_kib: parse_nonnegative(fields[3], "peak RSS")?,
        filesystem_input_operations: parse_nonnegative(fields[4], "filesystem input operations")?,
        filesystem_output_operations: parse_nonnegative(fields[5], "filesystem output operations")?,
    };
    if observation.wall_seconds <= 0.0 || observation.peak_rss_kib <= 0.0 {
        bail!("benchmark resource TSV lacks wall time or peak RSS");
    }
    Ok(observation)
}

pub(in crate::workflow) fn distribution(values: &[f64]) -> Result<BenchmarkMetricDistribution> {
    if values.len() < 2 || values.iter().any(|value| !value.is_finite() || *value < 0.0) {
        bail!("benchmark distribution observations are invalid");
    }
    let minimum = values.iter().copied().fold(f64::INFINITY, f64::min);
    let maximum = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    // Floating-point accumulation can place the mean infinitesimally outside the observed range.
    let mean = (values.iter().sum::<f64>() / values.len() as f64).clamp(minimum, maximum);
    let variance =
        values.iter().map(|value| (value - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
    Ok(BenchmarkMetricDistribution {
        minimum,
        maximum,
        mean,
        sample_standard_deviation: variance.sqrt(),
    })
}

fn parse_nonnegative(value: &str, field: &str) -> Result<f64> {
    let value = value.parse::<f64>().with_context(|| format!("parse benchmark {field}"))?;
    if !value.is_finite() || value < 0.0 {
        bail!("benchmark {field} is invalid");
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::distribution;

    #[test]
    fn distributions_use_sample_standard_deviation() -> anyhow::Result<()> {
        let observed = distribution(&[1.0, 2.0, 3.0])?;
        assert_eq!(observed.minimum, 1.0);
        assert_eq!(observed.maximum, 3.0);
        assert_eq!(observed.mean, 2.0);
        assert_eq!(observed.sample_standard_deviation, 1.0);
        Ok(())
    }

    #[test]
    fn identical_decimal_observations_remain_inside_the_observed_range() -> anyhow::Result<()> {
        let observed = distribution(&[0.36, 0.36, 0.36])?;
        assert_eq!(observed.minimum, 0.36);
        assert_eq!(observed.maximum, 0.36);
        assert_eq!(observed.mean, 0.36);
        assert_eq!(observed.sample_standard_deviation, 0.0);
        Ok(())
    }
}
