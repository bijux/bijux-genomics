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
