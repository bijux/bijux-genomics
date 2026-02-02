use bijux_domain_bam::metrics::BamMetricsV1;
use bijux_domain_bam::metrics::{evaluate_bam_invariants, BamInvariantThresholds};

pub(super) fn accounting_section(rows: &[bijux_core::FactsRowV1]) -> serde_json::Value {
    let mut stages = Vec::new();
    for row in rows {
        let reads = row.reads_out.or(row.reads_in);
        let bases = row.bases_out.or(row.bases_in);
        stages.push(serde_json::json!({
            "stage_id": row.stage_id,
            "tool_id": row.tool_id,
            "reads": reads,
            "bases": bases,
        }));
    }
    serde_json::json!({"stages": stages})
}

pub(super) fn bam_accounting_section(rows: &[bijux_core::FactsRowV1]) -> serde_json::Value {
    let mut entries = Vec::new();
    let thresholds = BamInvariantThresholds::default();
    for row in rows {
        if !row.stage_id.starts_with("bam.") {
            continue;
        }
        let metrics: BamMetricsV1 = match serde_json::from_value(row.metrics.clone()) {
            Ok(metrics) => metrics,
            Err(_) => BamMetricsV1::empty(),
        };
        let alignment = &metrics.alignment;
        let dup_fraction = if alignment.total > 0 {
            u64_to_f64(alignment.duplicates) / u64_to_f64(alignment.total)
        } else {
            0.0
        };
        let coverage_mean = metrics.coverage.mean;
        let complexity_reads = metrics.complexity.observed_reads;
        let verdict = metrics
            .stage_verdict
            .clone()
            .unwrap_or_else(|| {
                evaluate_bam_invariants(&row.stage_id, &metrics, &thresholds)
                    .verdict
                    .into()
            });
        entries.push(serde_json::json!({
            "stage_id": row.stage_id,
            "tool_id": row.tool_id,
            "reads_in": row.reads_in,
            "reads_out": row.reads_out,
            "duplicates_fraction": dup_fraction,
            "coverage_mean": coverage_mean,
            "complexity_observed_reads": complexity_reads,
            "verdict": verdict,
        }));
    }
    serde_json::json!({ "entries": entries })
}

pub(super) fn bam_findings_section(rows: &[bijux_core::FactsRowV1]) -> serde_json::Value {
    let mut findings = Vec::new();
    for row in rows {
        if !row.stage_id.starts_with("bam.") {
            continue;
        }
        let metrics: BamMetricsV1 = serde_json::from_value(row.metrics.clone())
            .unwrap_or_else(|_| BamMetricsV1::empty());
        let auth = metrics.authenticity.score;
        if auth >= 0.6 {
            findings.push(format!(
                "Sample shows authentic aDNA characteristics (authenticity score {auth:.2})."
            ));
        } else if auth > 0.0 {
            findings.push(format!(
                "Authenticity signal is weak (authenticity score {auth:.2}); review damage profile."
            ));
        }
        let dup_fraction = if metrics.alignment.total > 0 {
            u64_to_f64(metrics.alignment.duplicates) / u64_to_f64(metrics.alignment.total)
        } else {
            0.0
        };
        if dup_fraction >= 0.5 {
            findings.push("High duplication suggests low library complexity.".to_string());
        }
        if metrics.contamination.estimate >= 0.1 && metrics.damage.c_to_t_5p < 0.05 {
            findings.push("Contamination likely modern given low damage signal.".to_string());
        }
    }
    findings.truncate(5);
    serde_json::json!({ "findings": findings })
}

pub(super) fn bam_verdict_table(rows: &[bijux_core::FactsRowV1]) -> serde_json::Value {
    let thresholds = BamInvariantThresholds::default();
    let mut entries = Vec::new();
    for row in rows {
        if !row.stage_id.starts_with("bam.") {
            continue;
        }
        let metrics: BamMetricsV1 = serde_json::from_value(row.metrics.clone())
            .unwrap_or_else(|_| BamMetricsV1::empty());
        let verdict = metrics
            .stage_verdict
            .clone()
            .unwrap_or_else(|| {
                evaluate_bam_invariants(&row.stage_id, &metrics, &thresholds)
                    .verdict
                    .into()
            });
        let downstream_ok = metrics.coverage.mean >= 0.5 && metrics.coverage.breadth_1x >= 0.1;
        entries.push(serde_json::json!({
            "stage_id": row.stage_id,
            "tool_id": row.tool_id,
            "coverage": metrics.coverage.mean,
            "dup_fraction": if metrics.alignment.total > 0 {
                u64_to_f64(metrics.alignment.duplicates) / u64_to_f64(metrics.alignment.total)
            } else { 0.0 },
            "damage": metrics.damage.c_to_t_5p.max(metrics.damage.g_to_a_3p),
            "contamination": metrics.contamination.estimate,
            "sex_class": metrics.sex.classification,
            "verdict": verdict,
            "downstream_suitable": downstream_ok,
        }));
    }
    serde_json::json!({ "entries": entries })
}

pub(super) fn bam_plots_section(rows: &[bijux_core::FactsRowV1]) -> serde_json::Value {
    let mut plots = Vec::new();
    for row in rows {
        if !row.stage_id.starts_with("bam.") {
            continue;
        }
        let metrics: BamMetricsV1 = serde_json::from_value(row.metrics.clone())
            .unwrap_or_else(|_| BamMetricsV1::empty());
        plots.push(serde_json::json!({
            "stage_id": row.stage_id,
            "damage": {
                "c_to_t_5p": metrics.damage.c_to_t_5p,
                "g_to_a_3p": metrics.damage.g_to_a_3p
            },
            "fragment_length": {
                "mean": metrics.fragment_length.mean,
                "p10": metrics.fragment_length.p10,
                "p90": metrics.fragment_length.p90,
                "short_fraction": metrics.fragment_length.short_fraction
            },
            "coverage": {
                "mean": metrics.coverage.mean,
                "breadth_1x": metrics.coverage.breadth_1x,
                "breadth_3x": metrics.coverage.breadth_3x,
                "breadth_5x": metrics.coverage.breadth_5x
            },
            "dup_vs_complexity": {
                "dup_fraction": if metrics.alignment.total > 0 {
                    u64_to_f64(metrics.alignment.duplicates) / u64_to_f64(metrics.alignment.total)
                } else { 0.0 },
                "observed_reads": metrics.complexity.observed_reads
            }
        }));
    }
    serde_json::json!({ "entries": plots })
}

pub(super) fn impact_metrics_section(rows: &[bijux_core::FactsRowV1]) -> serde_json::Value {
    let mut impacts = Vec::new();
    for row in rows {
        if row.stage_id == "fastq.filter" || row.stage_id == "fastq.trim" {
            let reads_in = row.reads_in.unwrap_or(0);
            let reads_out = row.reads_out.unwrap_or(0);
            let bases_in = row.bases_in.unwrap_or(0);
            let bases_out = row.bases_out.unwrap_or(0);
            let read_drop = reads_in.saturating_sub(reads_out);
            let base_drop = bases_in.saturating_sub(bases_out);
            impacts.push(serde_json::json!({
                "stage_id": row.stage_id,
                "tool_id": row.tool_id,
                "reads_dropped": read_drop,
                "bases_dropped": base_drop,
            }));
        }
    }
    serde_json::json!({"impact": impacts})
}

pub(super) fn findings_section(rows: &[bijux_core::FactsRowV1]) -> serde_json::Value {
    let mut warnings: Vec<String> = Vec::new();
    let mut suspected = Vec::new();
    let mut recommendations = Vec::new();

    for row in rows {
        if row.stage_id == "fastq.qc_post" {
            let adapter_content = row
                .metrics
                .get("adapter_content_mean")
                .and_then(serde_json::Value::as_f64);
            if adapter_content.is_some_and(|v| v > 0.1) {
                suspected.push("adapter contamination detected".to_string());
                recommendations.push(
                    "enable adapter trimming or adjust adapter preset".to_string(),
                );
            }
            let duplication = row
                .metrics
                .get("duplication_rate")
                .and_then(serde_json::Value::as_f64);
            if duplication.is_some_and(|v| v > 0.5) {
                suspected.push("high duplication rate".to_string());
                recommendations.push(
                    "consider deduplication or library complexity checks".to_string(),
                );
            }
        }
        if row.stage_id == "fastq.merge" {
            let merge_rate = row.metrics.get("merge_rate").and_then(serde_json::Value::as_f64);
            if merge_rate.is_some_and(|v| v < 0.05) {
                suspected.push("low merge rate suggests long inserts".to_string());
                recommendations.push(
                    "disable merge stage or adjust overlap parameters".to_string(),
                );
            }
        }
        if row.stage_id == "fastq.filter" {
            if let (Some(ri), Some(ro)) = (row.reads_in, row.reads_out) {
                if ri > 0 && (u64_to_f64(ro) / u64_to_f64(ri)) < 0.5 {
                    suspected.push("high read loss during filtering".to_string());
                    recommendations.push(
                        "relax filtering thresholds or inspect input quality".to_string(),
                    );
                }
            }
        }
    }

    warnings.truncate(5);
    suspected.truncate(5);
    recommendations.truncate(5);

    serde_json::json!({
        "warnings": warnings,
        "suspected_issues": suspected,
        "recommendations": recommendations,
    })
}
