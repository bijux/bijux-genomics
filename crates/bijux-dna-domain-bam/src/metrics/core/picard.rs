use anyhow::Context;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InsertSizeMetricsV1 {
    pub median_insert_size: f64,
    pub mean_insert_size: f64,
    pub standard_deviation: f64,
    pub median_absolute_deviation: f64,
    pub min_insert_size: u64,
    pub max_insert_size: u64,
    pub read_pairs: u64,
    pub pair_orientation_fr_fraction: f64,
}

impl InsertSizeMetricsV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            median_insert_size: 0.0,
            mean_insert_size: 0.0,
            standard_deviation: 0.0,
            median_absolute_deviation: 0.0,
            min_insert_size: 0,
            max_insert_size: 0,
            read_pairs: 0,
            pair_orientation_fr_fraction: 0.0,
        }
    }
}

impl Default for InsertSizeMetricsV1 {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GcBiasMetricsV1 {
    #[serde(default, skip_serializing_if = "is_zero_f64")]
    pub gc_bias_score: f64,
    pub at_dropout: f64,
    pub gc_dropout: f64,
    pub total_clusters: u64,
    pub aligned_reads: u64,
    pub windows: u64,
    pub read_starts: u64,
}

impl GcBiasMetricsV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            gc_bias_score: 0.0,
            at_dropout: 0.0,
            gc_dropout: 0.0,
            total_clusters: 0,
            aligned_reads: 0,
            windows: 0,
            read_starts: 0,
        }
    }
}

impl Default for GcBiasMetricsV1 {
    fn default() -> Self {
        Self::empty()
    }
}

/// # Errors
/// Returns an error if the Picard insert-size metrics file cannot be parsed.
pub fn parse_picard_insert_size_metrics(
    path: &std::path::Path,
) -> anyhow::Result<InsertSizeMetricsV1> {
    let raw = std::fs::read_to_string(path).context("read picard insert-size metrics")?;
    let (header, data) = first_table_header_and_row(&raw)
        .ok_or_else(|| anyhow::anyhow!("insert-size metrics table missing"))?;
    let map = header_map(header, data);
    Ok(InsertSizeMetricsV1 {
        median_insert_size: f64_field(&map, "MEDIAN_INSERT_SIZE"),
        mean_insert_size: f64_field(&map, "MEAN_INSERT_SIZE"),
        standard_deviation: f64_field(&map, "STANDARD_DEVIATION"),
        median_absolute_deviation: f64_field(&map, "MEDIAN_ABSOLUTE_DEVIATION"),
        min_insert_size: u64_field(&map, "MIN_INSERT_SIZE"),
        max_insert_size: u64_field(&map, "MAX_INSERT_SIZE"),
        read_pairs: u64_field(&map, "READ_PAIRS"),
        pair_orientation_fr_fraction: pair_orientation_fr_fraction(&map),
    })
}

/// # Errors
/// Returns an error if the Picard GC-bias metrics file cannot be parsed.
pub fn parse_picard_gc_bias_metrics(path: &std::path::Path) -> anyhow::Result<GcBiasMetricsV1> {
    let raw = std::fs::read_to_string(path).context("read picard gc-bias metrics")?;
    let (header, data) = first_table_header_and_row(&raw)
        .ok_or_else(|| anyhow::anyhow!("gc-bias metrics table missing"))?;
    let map = header_map(header, data);
    let at_dropout = f64_field(&map, "AT_DROPOUT");
    let gc_dropout = f64_field(&map, "GC_DROPOUT");
    let gc_bias_score = normalize_dropout(at_dropout).max(normalize_dropout(gc_dropout));
    Ok(GcBiasMetricsV1 {
        gc_bias_score,
        at_dropout,
        gc_dropout,
        total_clusters: u64_field(&map, "TOTAL_CLUSTERS"),
        aligned_reads: u64_field(&map, "ALIGNED_READS"),
        windows: u64_field(&map, "WINDOWS"),
        read_starts: u64_field(&map, "READ_STARTS"),
    })
}

fn first_table_header_and_row(raw: &str) -> Option<(Vec<&str>, Vec<&str>)> {
    let lines: Vec<&str> = raw
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    lines.windows(2).find_map(|window| {
        let header: Vec<&str> = window[0].split('\t').collect();
        let data: Vec<&str> = window[1].split('\t').collect();
        (header.len() > 1 && data.len() > 1).then_some((header, data))
    })
}

fn header_map<'a>(
    header: Vec<&'a str>,
    data: Vec<&'a str>,
) -> std::collections::BTreeMap<&'a str, &'a str> {
    header.into_iter().zip(data).collect::<std::collections::BTreeMap<_, _>>()
}

fn f64_field(map: &std::collections::BTreeMap<&str, &str>, key: &str) -> f64 {
    map.get(key).and_then(|value| value.parse::<f64>().ok()).unwrap_or(0.0)
}

fn u64_field(map: &std::collections::BTreeMap<&str, &str>, key: &str) -> u64 {
    map.get(key).and_then(|value| value.parse::<u64>().ok()).unwrap_or(0)
}

fn pair_orientation_fr_fraction(map: &std::collections::BTreeMap<&str, &str>) -> f64 {
    let orientation =
        map.get("PAIR_ORIENTATION").copied().unwrap_or_default().trim().to_ascii_uppercase();
    if orientation == "FR" {
        1.0
    } else {
        0.0
    }
}

fn normalize_dropout(value: f64) -> f64 {
    if value > 1.0 {
        value / 100.0
    } else {
        value
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero_f64(value: &f64) -> bool {
    *value == 0.0
}

#[cfg(test)]
mod tests {
    use super::{parse_picard_gc_bias_metrics, parse_picard_insert_size_metrics};
    use anyhow::Result;
    use std::path::PathBuf;

    fn fixture(path: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../bijux-dna-stages-bam/tests/fixtures/observer/default")
            .join(path)
    }

    #[test]
    fn parse_insert_size_fixture() -> Result<()> {
        let metrics = parse_picard_insert_size_metrics(&fixture("insert_size.metrics.txt"))?;
        assert!(metrics.mean_insert_size > 0.0);
        assert!(metrics.read_pairs > 0);
        Ok(())
    }

    #[test]
    fn parse_gc_bias_fixture() -> Result<()> {
        let metrics = parse_picard_gc_bias_metrics(&fixture("gc_bias.metrics.txt"))?;
        assert!(metrics.total_clusters >= metrics.aligned_reads);
        assert!(metrics.windows > 0);
        Ok(())
    }
}
