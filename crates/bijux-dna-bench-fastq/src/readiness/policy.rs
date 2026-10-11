#[must_use]
pub(super) fn has_normalized_parser(value: &str) -> bool {
    matches!(value, "comparable" | "benchmark_normalized" | "parse_normalized")
}

pub(super) fn coverage_percent(covered: usize, total: usize) -> anyhow::Result<f64> {
    anyhow::ensure!(covered <= total, "FASTQ coverage exceeds the observed row count");
    let covered = u32::try_from(covered)?;
    let total = u32::try_from(total)?;
    Ok(if total == 0 { 0.0 } else { f64::from(covered) * 100.0 / f64::from(total) })
}
