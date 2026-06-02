use std::path::Path;

#[must_use]
pub fn collect_alignment_summary_metrics_args(
    bam: &Path,
    report: &Path,
) -> Vec<String> {
    vec![
        "picard".to_string(),
        "CollectAlignmentSummaryMetrics".to_string(),
        format!("I={}", bam.display()),
        format!("O={}", report.display()),
        "VALIDATION_STRINGENCY=SILENT".to_string(),
    ]
}

#[must_use]
pub fn collect_insert_size_metrics_args(
    bam: &Path,
    report: &Path,
    histogram: &Path,
) -> Vec<String> {
    vec![
        "picard".to_string(),
        "CollectInsertSizeMetrics".to_string(),
        format!("I={}", bam.display()),
        format!("O={}", report.display()),
        format!("H={}", histogram.display()),
        "M=0.5".to_string(),
    ]
}

#[must_use]
pub fn collect_gc_bias_metrics_args(
    bam: &Path,
    reference: &Path,
    report: &Path,
    summary: &Path,
    chart: &Path,
) -> Vec<String> {
    vec![
        "picard".to_string(),
        "CollectGcBiasMetrics".to_string(),
        format!("I={}", bam.display()),
        format!("R={}", reference.display()),
        format!("O={}", report.display()),
        format!("S={}", summary.display()),
        format!("CHART={}", chart.display()),
    ]
}
