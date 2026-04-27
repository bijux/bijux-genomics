use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use bijux_dna_domain_fastq::metrics::{
    FastqDeltaMetricsV1, FastqFilterMetricsV1, RetentionReportMetricV1,
};

#[derive(Debug, Default, Clone)]
pub(super) struct FilterRemovalCounts {
    pub n: u64,
    pub entropy: u64,
    pub low_complexity: u64,
    pub kmer: u64,
    pub contaminant_kmer: u64,
    pub length: u64,
}

pub(super) fn filter_removals_from_fastp(path: &Path) -> Option<FilterRemovalCounts> {
    let raw = std::fs::read_to_string(path).ok()?;
    let parsed: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let filtering = parsed.get("filtering_result")?;
    let by_n = filtering.get("too_many_N_reads").and_then(serde_json::Value::as_u64).unwrap_or(0);
    let by_entropy =
        filtering.get("low_complexity_reads").and_then(serde_json::Value::as_u64).unwrap_or(0);
    let by_length =
        filtering.get("too_short_reads").and_then(serde_json::Value::as_u64).unwrap_or(0)
            + filtering.get("too_long_reads").and_then(serde_json::Value::as_u64).unwrap_or(0);
    Some(FilterRemovalCounts {
        n: by_n,
        entropy: by_entropy,
        low_complexity: by_entropy,
        kmer: 0,
        contaminant_kmer: 0,
        length: by_length,
    })
}

pub(super) fn filter_removals_from_bbduk_stats(
    path: &Path,
    kmer_ref_used: bool,
) -> Option<FilterRemovalCounts> {
    let raw = std::fs::read_to_string(path).ok()?;
    let mut removed = None;
    for line in raw.lines() {
        let line = line.trim();
        if line.starts_with("Reads Removed") || line.starts_with("Reads removed") {
            let digits: String = line.chars().filter(char::is_ascii_digit).collect();
            if !digits.is_empty() {
                removed = digits.parse::<u64>().ok();
            }
        }
        if removed.is_none() && (line.starts_with("#Matched") || line.starts_with("Matched")) {
            if let Some(field) = line.split_whitespace().nth(1) {
                let digits: String = field.chars().filter(char::is_ascii_digit).collect();
                if !digits.is_empty() {
                    removed = digits.parse::<u64>().ok();
                }
            }
        }
    }
    let removed = removed?;
    Some(FilterRemovalCounts {
        n: 0,
        entropy: 0,
        low_complexity: 0,
        kmer: if kmer_ref_used { removed } else { 0 },
        contaminant_kmer: if kmer_ref_used { removed } else { 0 },
        length: 0,
    })
}

pub(super) fn filter_metrics_with_removals(
    stage_id: &bijux_dna_core::ids::StageId,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
    params: &serde_json::Value,
    effective_params: &serde_json::Value,
    removals: &FilterRemovalCounts,
) -> Result<serde_json::Value> {
    let stats = super::stats_for_paths(&[
        inputs.first().map(PathBuf::as_path),
        outputs.first().map(PathBuf::as_path),
    ])?;
    let input = stats.first().copied().unwrap_or_else(super::zero_seqkit_metrics);
    let output = stats.get(1).copied().unwrap_or_else(super::zero_seqkit_metrics);
    let (pairs_in, pairs_out) = super::pair_counts_from_paths(inputs, outputs)?;
    let read_retention = if input.reads > 0 {
        super::f64_from_u64(output.reads) / super::f64_from_u64(input.reads)
    } else {
        0.0
    };
    let base_retention = if input.bases > 0 {
        super::f64_from_u64(output.bases) / super::f64_from_u64(input.bases)
    } else {
        0.0
    };
    let delta = FastqDeltaMetricsV1 {
        read_retention,
        base_retention,
        mean_q_delta: output.mean_q - input.mean_q,
        gc_delta: output.gc_percent - input.gc_percent,
    };
    let retention = RetentionReportMetricV1 {
        value: read_retention,
        numerator_reads: output.reads,
        denominator_reads: input.reads,
        numerator_bases: output.bases,
        denominator_bases: input.bases,
        definition: "reads_out / reads_in".to_string(),
        stage_boundary: stage_id.to_string(),
        conditions: super::retention_conditions_from_effective(stage_id, effective_params, params),
    };
    Ok(serde_json::to_value(FastqFilterMetricsV1 {
        reads_in: input.reads,
        reads_out: output.reads,
        reads_dropped: input.reads.saturating_sub(output.reads),
        reads_removed_by_n: removals.n,
        reads_removed_by_entropy: removals.entropy,
        reads_removed_low_complexity: removals.low_complexity,
        reads_removed_by_kmer: removals.kmer,
        reads_removed_contaminant_kmer: removals.contaminant_kmer,
        reads_removed_by_length: removals.length,
        bases_in: input.bases,
        bases_out: output.bases,
        pairs_in,
        pairs_out,
        mean_q_before: input.mean_q,
        mean_q_after: output.mean_q,
        delta_metrics: delta,
        retention,
    })?)
}

pub(super) fn parse_screen_report(path: &Path) -> Result<(f64, serde_json::Value)> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("screen report missing: {}", path.display()))?;
    let mut entries = Vec::new();
    let mut unmapped_percent = None;
    let mut errors = Vec::new();
    for (idx, line) in raw.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 3 {
            errors.push(format!("line {} has {} columns", idx + 1, parts.len()));
            continue;
        }
        let label = parts[0].trim().to_string();
        let percent_col = parts
            .last()
            .ok_or_else(|| anyhow!("screen report line {} missing percent", idx + 1))?;
        let percent_str = percent_col.trim().trim_end_matches('%');
        let percent = percent_str
            .parse::<f64>()
            .with_context(|| format!("screen report line {} percent parse", idx + 1))?;
        let label_lower = label.to_lowercase();
        if label_lower.contains("unmapped")
            || (label_lower.contains("no hit") && unmapped_percent.is_none())
        {
            unmapped_percent = Some(percent);
        }
        entries.push(serde_json::json!({
            "reference": label,
            "percent": percent,
        }));
    }
    if !errors.is_empty() {
        return Err(anyhow!("screen report parse errors: {}", errors.join("; ")));
    }
    if entries.is_empty() {
        return Ok((
            0.0,
            serde_json::json!({
                "schema_version": "bijux.screen_summary.v1",
                "entries": entries,
                "warning": "empty_report",
            }),
        ));
    }
    let contamination_rate = unmapped_percent.map_or(0.0, |value| (100.0 - value).max(0.0) / 100.0);
    Ok((
        contamination_rate,
        serde_json::json!({
            "schema_version": "bijux.screen_summary.v1",
            "entries": entries,
        }),
    ))
}
