fn read_json_value(path: &Path) -> Option<serde_json::Value> {
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
}

pub(super) fn pipeline_verdict_from_rows(rows: &[FactsRowV1]) -> PipelineVerdictV1 {
    let mut verdict = InvariantStatusV1::Pass;
    let mut reasons = Vec::new();
    for row in rows {
        let Some(stage_report_path) = report_path_for(&row.reports, "stage_report") else {
            continue;
        };
        let Some(stage_report_value) = read_json_value(Path::new(&stage_report_path)) else {
            continue;
        };
        let Ok(report) = serde_json::from_value::<StageReportV1>(stage_report_value) else {
            continue;
        };
        let Some(stage_verdict) = report.verdict else {
            continue;
        };
        verdict = std::cmp::max(verdict, stage_verdict.verdict.clone());
        if stage_verdict.verdict != InvariantStatusV1::Pass {
            reasons.push(format!(
                "{}:{:?}",
                stage_verdict.stage_id, stage_verdict.verdict
            ));
        }
    }
    PipelineVerdictV1 { verdict, reasons }
}

pub(super) fn pipeline_verdict_section(rows: &[FactsRowV1]) -> serde_json::Value {
    let verdict = pipeline_verdict_from_rows(rows);
    serde_json::json!({
        "verdict": verdict.verdict,
        "reasons": verdict.reasons,
        "aggregation_policy": {
            "schema_version": "bijux.pipeline_verdict_policy.v1",
            "rule": "worst_status",
            "pass_condition": "all stages pass",
            "warn_condition": "any stage warn and no stage fail",
            "fail_condition": "any stage fail",
        }
    })
}

pub(super) fn comparison_view_section(rows: &[FactsRowV1]) -> serde_json::Value {
    let mut by_stage: BTreeMap<String, Vec<&FactsRowV1>> = BTreeMap::new();
    for row in rows {
        by_stage.entry(row.stage_id.clone()).or_default().push(row);
    }
    let mut stages = Vec::new();
    for (stage_id, entries) in by_stage {
        if entries.len() < 2 {
            continue;
        }
        let mut tools = Vec::new();
        let mut rank_inputs = Vec::new();
        for row in entries {
            let read_retention = match (row.reads_in, row.reads_out) {
                #[allow(clippy::cast_precision_loss)]
                (Some(ri), Some(ro)) if ri > 0 => Some(ro as f64 / ri as f64),
                _ => None,
            };
            let base_retention = match (row.bases_in, row.bases_out) {
                #[allow(clippy::cast_precision_loss)]
                (Some(bi), Some(bo)) if bi > 0 => Some(bo as f64 / bi as f64),
                _ => None,
            };
            let error_reduction_proxy = row
                .metrics
                .get("mean_q_delta")
                .and_then(serde_json::Value::as_f64);
            rank_inputs.push(RankInput {
                tool: row.tool_id.clone(),
                runtime_s: row.runtime_s,
                memory_mb: row.memory_mb,
                read_retention,
                base_retention,
                error_reduction_proxy,
            });

            let stage_report = stage_report_for_row(row);
            let verdict = stage_report
                .as_ref()
                .and_then(|report| report.verdict.as_ref())
                .map(|verdict| verdict.verdict.clone());
            let mut notes = Vec::new();
            if let Some(report) = stage_report.as_ref() {
                if let Some(verdict) = report.verdict.as_ref() {
                    notes.extend(verdict.reasons.clone());
                }
                notes.extend(report.warnings.clone());
                notes.extend(report.errors.clone());
            }
            let key_params = stage_report
                .as_ref()
                .and_then(tool_invocation_for_stage)
                .map_or_else(
                    || serde_json::json!({}),
                    |invocation| {
                        let params = if invocation.effective_params_json_normalized.is_null() {
                            invocation.parameters_json_normalized
                        } else {
                            invocation.effective_params_json_normalized
                        };
                        params_excerpt(&params, 8)
                    },
                );

            tools.push(serde_json::json!({
                "tool_id": row.tool_id,
                "tool_version": row.tool_version,
                "params_hash": row.params_hash,
                "runtime_s": row.runtime_s,
                "memory_mb": row.memory_mb,
                "read_retention": read_retention,
                "base_retention": base_retention,
                "verdict": verdict,
                "key_params": key_params,
                "notes": notes,
            }));
        }

        let recommended = build_rankings(&rank_inputs)
            .ok()
            .and_then(|rankings| rankings.get("BalancedPareto").cloned())
            .and_then(|entries| entries.first().map(|entry| entry.tool.clone()));
        stages.push(serde_json::json!({
            "stage_id": stage_id,
            "tools": tools,
            "recommended_default": recommended,
        }));
    }
    serde_json::json!({
        "stages": stages,
    })
}

pub(super) fn failure_hints_section(rows: &[FactsRowV1]) -> serde_json::Value {
    let mut failures: Vec<BenchmarkFailure> = Vec::new();
    for row in rows {
        if let Some(failures_value) = row.reports.get("failures") {
            if let Some(array) = failures_value.as_array() {
                for entry in array {
                    if let Ok(raw) = serde_json::from_value::<RawFailure>(entry.clone()) {
                        failures.push(classify_raw_failure(&raw));
                    }
                }
            }
        }
    }
    serde_json::json!({
        "failures": failures,
        "count": failures.len(),
    })
}

pub(super) fn read_tool_invocation(path: &Path) -> Option<ToolInvocationV1> {
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
}

pub(super) fn params_excerpt(value: &serde_json::Value, limit: usize) -> serde_json::Value {
    let Some(obj) = value.as_object() else {
        return value.clone();
    };
    let mut keys: Vec<_> = obj.keys().cloned().collect();
    keys.sort();
    let mut out = serde_json::Map::new();
    for key in keys.into_iter().take(limit) {
        if let Some(v) = obj.get(&key) {
            out.insert(key, v.clone());
        }
    }
    serde_json::Value::Object(out)
}

pub(super) fn stage_report_for_row(row: &FactsRowV1) -> Option<StageReportV1> {
    let path = report_path_for(&row.reports, "stage_report")?;
    let report_raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&report_raw).ok()
}

fn tool_invocation_for_stage(report: &StageReportV1) -> Option<ToolInvocationV1> {
    let invocation_raw = fs::read_to_string(&report.tool_invocation_path).ok()?;
    serde_json::from_str(&invocation_raw).ok()
}
