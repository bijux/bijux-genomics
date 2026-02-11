fn qc_delta_section(rows: &[FactsRowV1]) -> serde_json::Value {
    let mut validate_mean_q = None;
    let mut qc_post_mean_q = None;
    for row in rows {
        if row.stage_id == "fastq.validate_pre" {
            validate_mean_q = row
                .metrics
                .get("mean_q")
                .and_then(serde_json::Value::as_f64);
        }
        if row.stage_id == "fastq.qc_post" {
            qc_post_mean_q = row
                .metrics
                .get("mean_q")
                .and_then(serde_json::Value::as_f64);
        }
    }
    let delta = match (validate_mean_q, qc_post_mean_q) {
        (Some(a), Some(b)) => Some(b - a),
        _ => None,
    };
    serde_json::json!({
        "validate_pre_mean_q": validate_mean_q,
        "qc_post_mean_q": qc_post_mean_q,
        "mean_q_delta": delta,
    })
}

fn contaminant_summary_section(rows: &[FactsRowV1]) -> serde_json::Value {
    let mut summary = None;
    let mut reads_removed = None;
    let mut percent_removed = None;
    let mut kmer_removed = None;
    let mut kmer_percent = None;
    for row in rows {
        if row.stage_id != "fastq.screen" {
            if row.stage_id == "fastq.filter" {
                if let Some(path) = report_path_for(&row.reports, "filter_report") {
                    if let Some(report) = read_json_value(Path::new(&path))
                        .and_then(|value| serde_json::from_value::<FilterReportV1>(value).ok())
                    {
                        kmer_removed = Some(report.reads_removed_contaminant_kmer);
                        if report.reads_in > 0 {
                            kmer_percent = Some(
                                u64_to_f64(report.reads_removed_contaminant_kmer)
                                    / u64_to_f64(report.reads_in),
                            );
                        }
                    }
                }
            }
            continue;
        }
        let reads_in = row.reads_in.unwrap_or(0);
        let reads_out = row.reads_out.unwrap_or(0);
        if reads_in > 0 && reads_out <= reads_in {
            reads_removed = Some(reads_in - reads_out);
            percent_removed = Some(u64_to_f64(reads_in - reads_out) / u64_to_f64(reads_in));
        }
        summary = row
            .metrics
            .get("contamination_summary")
            .cloned()
            .or_else(|| row.metrics.get("contamination_summary").cloned());
        break;
    }
    serde_json::json!({
        "reads_removed": reads_removed,
        "percent_removed": percent_removed,
        "kmer_reads_removed": kmer_removed,
        "kmer_percent_removed": kmer_percent,
        "top_taxa": summary.unwrap_or_else(|| serde_json::json!({})),
    })
}

fn read_json_value(path: &Path) -> Option<serde_json::Value> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
}

fn stage_report_fields(report: Option<&StageReportV1>) -> (String, String, String) {
    report.map_or_else(
        || (String::new(), String::new(), String::new()),
        |report| {
            (
                report.metrics_path.clone(),
                report.tool_invocation_path.clone(),
                report.effective_config_path.clone(),
            )
        },
    )
}

fn retention_context_from_report(
    path: Option<&str>,
) -> Option<(RetentionContextV1, RetentionDefinitionV1)> {
    let report = path
        .and_then(|path| read_json_value(Path::new(path)))
        .and_then(|value| serde_json::from_value::<RetentionReportV1>(value).ok())?;
    let definition = format!(
        "{} / {} ({})",
        serde_json::to_string(&report.numerator).unwrap_or_else(|_| "unknown".to_string()),
        serde_json::to_string(&report.denominator).unwrap_or_else(|_| "unknown".to_string()),
        report.units
    );
    let conditions = report.condition.clone();
    let context = RetentionContextV1 {
        stage_id: report.stage_id,
        tool_id: report.tool_id,
        definition,
        conditions,
    };
    let definition = RetentionDefinitionV1 {
        stage_id: context.stage_id.clone(),
        tool_id: context.tool_id.clone(),
        numerator: serde_json::to_string(&report.numerator)
            .unwrap_or_else(|_| "unknown".to_string()),
        denominator: serde_json::to_string(&report.denominator)
            .unwrap_or_else(|_| "unknown".to_string()),
        conditions: context.conditions.clone(),
    };
    Some((context, definition))
}

fn banks_from_report(path: Option<&str>, fallback: serde_json::Value) -> serde_json::Value {
    path.and_then(|path| read_json_value(Path::new(path)))
        .and_then(|value| value.get("banks").cloned())
        .unwrap_or(fallback)
}

fn telemetry_path_from_stage_report(path: Option<&str>) -> Option<String> {
    path.and_then(|path| {
        Path::new(path).parent().map(|parent| {
            let v2 = parent.join("telemetry.jsonl");
            if v2.exists() {
                v2.display().to_string()
            } else {
                parent
                    .join("telemetry")
                    .join("events.jsonl")
                    .display()
                    .to_string()
            }
        })
    })
}

fn telemetry_counts(paths: &[String]) -> (usize, usize) {
    let mut total_events = 0usize;
    let mut error_events = 0usize;
    for path in paths {
        let Ok(raw) = std::fs::read_to_string(path) else {
            continue;
        };
        for line in raw.lines() {
            if line.trim().is_empty() {
                continue;
            }
            total_events += 1;
            if let Ok(event) = serde_json::from_str::<TelemetryEventV1>(line) {
                if matches!(
                    event.event_name,
                    bijux_dna_runtime::TelemetryEventName::Error
                        | bijux_dna_runtime::TelemetryEventName::RunFailed
                ) || event.status == "error"
                {
                    error_events += 1;
                }
            }
        }
    }
    (total_events, error_events)
}

fn telemetry_timeline_from_paths(paths: &[String]) -> Vec<serde_json::Value> {
    let mut out = Vec::new();
    for path in paths {
        let Ok(raw) = std::fs::read_to_string(path) else {
            continue;
        };
        for line in raw.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let Ok(event) = serde_json::from_str::<TelemetryEventV1>(line) else {
                continue;
            };
            out.push(serde_json::json!({
                "timestamp": event.timestamp,
                "stage_id": event.stage_id,
                "tool_id": event.tool_id,
                "event": event.event_name,
                "status": event.status,
                "failure_code": event.failure_code,
            }));
        }
    }
    out.sort_by_key(std::string::ToString::to_string);
    out
}

fn telemetry_decisions_from_paths(
    paths: &[String],
) -> std::collections::BTreeMap<String, Vec<serde_json::Value>> {
    let mut by_stage: std::collections::BTreeMap<String, Vec<serde_json::Value>> =
        std::collections::BTreeMap::new();
    for path in paths {
        let Ok(raw) = std::fs::read_to_string(path) else {
            continue;
        };
        for line in raw.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let Ok(event) = serde_json::from_str::<TelemetryEventV1>(line) else {
                continue;
            };
            if !matches!(
                event.event_name,
                bijux_dna_runtime::TelemetryEventName::MergeDecision
                    | bijux_dna_runtime::TelemetryEventName::AdapterValidation
                    | bijux_dna_runtime::TelemetryEventName::ContaminantAction
                    | bijux_dna_runtime::TelemetryEventName::QualityGate
            ) {
                continue;
            }
            by_stage
                .entry(event.stage_id.clone())
                .or_default()
                .push(serde_json::json!({
                    "event": event.event_name,
                    "status": event.status,
                    "attrs": event.attrs,
                }));
        }
    }
    by_stage
}
