use std::path::Path;

use bijux_dna_domain_bam::params::MarkDupEffectiveParams;

#[must_use]
pub fn collect_alignment_summary_metrics_args(bam: &Path, report: &Path) -> Vec<String> {
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

#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn markdup_args_with_audit(
    bam: &Path,
    out_bam: &Path,
    flagstat_before: &Path,
    flagstat_after: &Path,
    idxstats_before: &Path,
    idxstats_after: &Path,
    summary: &Path,
    params: &MarkDupEffectiveParams,
) -> Vec<String> {
    let remove =
        matches!(params.duplicate_action, bijux_dna_domain_bam::params::DuplicateAction::Remove);
    let metrics = out_bam.with_file_name("markdup.metrics.txt");
    let command = format!(
        "samtools flagstat {bam} > {flagstat_before} && \
samtools idxstats {bam} > {idxstats_before} && \
picard MarkDuplicates I={bam} O={out} M={metrics} VALIDATION_STRINGENCY=SILENT ASSUME_SORTED=true REMOVE_DUPLICATES={remove} && \
samtools index {out} {out}.bai && \
samtools flagstat {out} > {flagstat_after} && \
samtools idxstats {out} > {idxstats_after} && \
python - <<'PY' {metrics} > {summary}\nimport json,sys\npayload={{\"input_bam\": \"{bam}\", \"output_bam\": \"{out}\", \"metrics\": sys.argv[1], \"remove_duplicates\": {remove}, \"tool\": \"picard\", \"artifacts\": {{\"flagstat_before\": \"{flagstat_before}\", \"flagstat_after\": \"{flagstat_after}\", \"idxstats_before\": \"{idxstats_before}\", \"idxstats_after\": \"{idxstats_after}\"}}}}\nprint(json.dumps(payload, indent=2))\nPY",
        bam = bam.display(),
        out = out_bam.display(),
        metrics = metrics.display(),
        remove = if remove { "true" } else { "false" },
        flagstat_before = flagstat_before.display(),
        flagstat_after = flagstat_after.display(),
        idxstats_before = idxstats_before.display(),
        idxstats_after = idxstats_after.display(),
        summary = summary.display(),
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
