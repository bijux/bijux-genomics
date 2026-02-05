#[must_use]
#[allow(clippy::too_many_lines)]
pub fn evaluate_invariants(
    stage_id: &StageId,
    metrics_json: &serde_json::Value,
    effective_params: &serde_json::Value,
    thresholds: &InvariantThresholds,
) -> InvariantEvaluation {
    let mut results = Vec::new();
    let parsed_params = parse_effective_params(stage_id, effective_params);
    if let Some(params) = parsed_params.as_ref() {
        let missing = params.missing_required_fields();
        if missing.is_empty() {
            results.push(result(
                "effective_params_present",
                InvariantStatusV1::Pass,
                "effective params present with required fields".to_string(),
                None,
            ));
        } else {
            results.push(result(
                "effective_params_present",
                InvariantStatusV1::Fail,
                format!("missing effective params fields: {}", missing.join(", ")),
                Some("populate canonical effective params for this stage".to_string()),
            ));
        }
    } else {
        results.push(result(
            "effective_params_present",
            InvariantStatusV1::Fail,
            "effective params missing or malformed".to_string(),
            Some("emit canonical effective params JSON in stage planner".to_string()),
        ));
    }

    let mut key_metrics = serde_json::Map::new();

    if stage_id == &STAGE_TRIM {
        if let Ok(metrics) = serde_json::from_value::<FastqTrimMetricsV1>(metrics_json.clone()) {
            key_metrics.insert(
                "read_retention".to_string(),
                serde_json::Value::from(metrics.delta_metrics.read_retention),
            );
            key_metrics.insert(
                "base_retention".to_string(),
                serde_json::Value::from(metrics.delta_metrics.base_retention),
            );
            key_metrics.insert(
                "mean_q_delta".to_string(),
                serde_json::Value::from(metrics.delta_metrics.mean_q_delta),
            );
            let (warn, fail) = retention_thresholds_for(parsed_params.as_ref(), thresholds);
            let retention = metrics.delta_metrics.read_retention;
            let remediation =
                Some("review min_len/quality thresholds or input quality".to_string());
            if retention < fail {
                results.push(result(
                    "retention_sanity",
                    InvariantStatusV1::Fail,
                    format!("read_retention {retention:.2} below fail threshold {fail:.2}"),
                    remediation,
                ));
            } else if retention < warn {
                results.push(result(
                    "retention_sanity",
                    InvariantStatusV1::Warn,
                    format!("read_retention {retention:.2} below warn threshold {warn:.2}"),
                    remediation,
                ));
            } else {
                results.push(result(
                    "retention_sanity",
                    InvariantStatusV1::Pass,
                    "read_retention within expected range".to_string(),
                    None,
                ));
            }
            let q_delta = metrics.delta_metrics.mean_q_delta;
            if q_delta < thresholds.mean_q_delta_fail {
                results.push(result(
                    "quality_direction",
                    InvariantStatusV1::Fail,
                    format!(
                        "mean_q_delta {:.2} below fail threshold {:.2}",
                        q_delta, thresholds.mean_q_delta_fail
                    ),
                    Some("review trimming settings; quality should not drop sharply".to_string()),
                ));
            } else if q_delta < thresholds.mean_q_delta_warn {
                results.push(result(
                    "quality_direction",
                    InvariantStatusV1::Warn,
                    format!(
                        "mean_q_delta {:.2} below warn threshold {:.2}",
                        q_delta, thresholds.mean_q_delta_warn
                    ),
                    Some("review trimming settings for quality regression".to_string()),
                ));
            } else {
                results.push(result(
                    "quality_direction",
                    InvariantStatusV1::Pass,
                    "mean_q_delta within expected bounds".to_string(),
                    None,
                ));
            }
        } else {
            results.push(result(
                "metrics_parse",
                InvariantStatusV1::Fail,
                "failed to parse trim metrics".to_string(),
                Some("verify metrics schema for fastq.trim".to_string()),
            ));
        }
    } else if stage_id == &STAGE_FILTER {
        if let Ok(metrics) = serde_json::from_value::<FastqFilterMetricsV1>(metrics_json.clone()) {
            key_metrics.insert(
                "read_retention".to_string(),
                serde_json::Value::from(metrics.delta_metrics.read_retention),
            );
            key_metrics.insert(
                "base_retention".to_string(),
                serde_json::Value::from(metrics.delta_metrics.base_retention),
            );
            key_metrics.insert(
                "mean_q_delta".to_string(),
                serde_json::Value::from(metrics.delta_metrics.mean_q_delta),
            );
            let (warn, fail) = retention_thresholds_for(parsed_params.as_ref(), thresholds);
            let retention = metrics.delta_metrics.read_retention;
            let remediation = Some("review filtering thresholds or input quality".to_string());
            if retention < fail {
                results.push(result(
                    "retention_sanity",
                    InvariantStatusV1::Fail,
                    format!("read_retention {retention:.2} below fail threshold {fail:.2}"),
                    remediation,
                ));
            } else if retention < warn {
                results.push(result(
                    "retention_sanity",
                    InvariantStatusV1::Warn,
                    format!("read_retention {retention:.2} below warn threshold {warn:.2}"),
                    remediation,
                ));
            } else {
                results.push(result(
                    "retention_sanity",
                    InvariantStatusV1::Pass,
                    "read_retention within expected range".to_string(),
                    None,
                ));
            }
            let q_delta = metrics.delta_metrics.mean_q_delta;
            if q_delta < thresholds.mean_q_delta_fail {
                results.push(result(
                    "quality_direction",
                    InvariantStatusV1::Fail,
                    format!(
                        "mean_q_delta {:.2} below fail threshold {:.2}",
                        q_delta, thresholds.mean_q_delta_fail
                    ),
                    Some("review filtering settings for quality regression".to_string()),
                ));
            } else if q_delta < thresholds.mean_q_delta_warn {
                results.push(result(
                    "quality_direction",
                    InvariantStatusV1::Warn,
                    format!(
                        "mean_q_delta {:.2} below warn threshold {:.2}",
                        q_delta, thresholds.mean_q_delta_warn
                    ),
                    Some("review filtering settings for quality regression".to_string()),
                ));
            } else {
                results.push(result(
                    "quality_direction",
                    InvariantStatusV1::Pass,
                    "mean_q_delta within expected bounds".to_string(),
                    None,
                ));
            }
            if metrics.reads_in > 0 {
                #[allow(clippy::cast_precision_loss)]
                let n_rate = metrics.reads_removed_by_n as f64 / metrics.reads_in as f64;
                key_metrics.insert("n_rate".to_string(), serde_json::Value::from(n_rate));
                if n_rate > thresholds.n_rate_fail {
                    results.push(result(
                        "n_rate_sanity",
                        InvariantStatusV1::Fail,
                        format!(
                            "n_rate {:.3} exceeds fail threshold {:.3}",
                            n_rate, thresholds.n_rate_fail
                        ),
                        Some("review N filtering thresholds and input quality".to_string()),
                    ));
                } else if n_rate > thresholds.n_rate_warn {
                    results.push(result(
                        "n_rate_sanity",
                        InvariantStatusV1::Warn,
                        format!(
                            "n_rate {:.3} exceeds warn threshold {:.3}",
                            n_rate, thresholds.n_rate_warn
                        ),
                        Some("review N filtering thresholds and input quality".to_string()),
                    ));
                } else {
                    results.push(result(
                        "n_rate_sanity",
                        InvariantStatusV1::Pass,
                        "n_rate within expected bounds".to_string(),
                        None,
                    ));
                }
            }
        } else {
            results.push(result(
                "metrics_parse",
                InvariantStatusV1::Fail,
                "failed to parse filter metrics".to_string(),
                Some("verify metrics schema for fastq.filter".to_string()),
            ));
        }
    } else if stage_id == &STAGE_VALIDATE_PRE {
        if let Ok(metrics) =
            serde_json::from_value::<FastqValidateMetricsV1>(metrics_json.clone())
        {
            key_metrics.insert(
                "reads_invalid".to_string(),
                serde_json::Value::from(metrics.reads_invalid),
            );
            if metrics.reads_invalid > 0 {
                results.push(result(
                    "validate_malformed_reads",
                    InvariantStatusV1::Fail,
                    format!("reads_invalid {} > 0", metrics.reads_invalid),
                    Some("inspect input FASTQ integrity and re-run validation".to_string()),
                ));
            } else {
                results.push(result(
                    "validate_malformed_reads",
                    InvariantStatusV1::Pass,
                    "no malformed reads detected".to_string(),
                    None,
                ));
            }
        } else {
            results.push(result(
                "metrics_parse",
                InvariantStatusV1::Fail,
                "failed to parse validate metrics".to_string(),
                Some("verify metrics schema for fastq.validate_pre".to_string()),
            ));
        }
    } else if stage_id == &STAGE_MERGE {
        if let Ok(metrics) = serde_json::from_value::<FastqMergeMetricsV1>(metrics_json.clone()) {
            key_metrics.insert(
                "merge_rate".to_string(),
                serde_json::Value::from(metrics.merge_rate),
            );
            let rate = metrics.merge_rate;
            if rate < thresholds.merge_rate_fail_low || rate > thresholds.merge_rate_fail_high {
                results.push(result(
                    "merge_rate_range",
                    InvariantStatusV1::Fail,
                    format!(
                        "merge_rate {:.3} outside fail range [{:.2}, {:.2}]",
                        rate, thresholds.merge_rate_fail_low, thresholds.merge_rate_fail_high
                    ),
                    Some("inspect merge parameters and read overlap suitability".to_string()),
                ));
            } else if rate < thresholds.merge_rate_warn_low
                || rate > thresholds.merge_rate_warn_high
            {
                results.push(result(
                    "merge_rate_range",
                    InvariantStatusV1::Warn,
                    format!(
                        "merge_rate {:.3} outside warn range [{:.2}, {:.2}]",
                        rate, thresholds.merge_rate_warn_low, thresholds.merge_rate_warn_high
                    ),
                    Some("inspect merge parameters and read overlap suitability".to_string()),
                ));
            } else {
                results.push(result(
                    "merge_rate_range",
                    InvariantStatusV1::Pass,
                    "merge_rate within expected range".to_string(),
                    None,
                ));
            }
        } else {
            results.push(result(
                "metrics_parse",
                InvariantStatusV1::Fail,
                "failed to parse merge metrics".to_string(),
                Some("verify metrics schema for fastq.merge".to_string()),
            ));
        }
    }

    let mut verdict = InvariantStatusV1::Pass;
    let mut reasons = Vec::new();
    for entry in &results {
        verdict = worst_status(verdict, &entry.status);
        if entry.status != InvariantStatusV1::Pass {
            reasons.push(entry.id.clone());
        }
    }

    let stage_verdict = StageVerdictV1 {
        stage_id: stage_id.to_string(),
        verdict,
        reasons,
        key_metrics: serde_json::Value::Object(key_metrics),
    };

    InvariantEvaluation {
        results,
        verdict: stage_verdict,
    }
}

#[allow(dead_code)]
pub const CORE_STAGES: [StageId; 6] = [
    STAGE_VALIDATE_PRE,
    STAGE_TRIM,
    STAGE_MERGE,
    STAGE_CORRECT,
    STAGE_FILTER,
    STAGE_STATS_NEUTRAL,
];

#[allow(dead_code)]
pub const OPTIONAL_STAGES: [StageId; 3] = [STAGE_QC_POST, STAGE_UMI, STAGE_SCREEN];

#[allow(dead_code)]
pub const META_STAGES: [StageId; 1] = [STAGE_PREPROCESS];

#[allow(dead_code)]
pub const MUTATING_STAGES: [StageId; 5] = [
    STAGE_TRIM,
    STAGE_MERGE,
    STAGE_CORRECT,
    STAGE_FILTER,
    STAGE_UMI,
];

#[allow(dead_code)]
pub const LOSSLESS_STAGES: [StageId; 2] = [STAGE_VALIDATE_PRE, STAGE_STATS_NEUTRAL];

#[allow(dead_code)]
pub const OBSERVATIONAL_STAGES: [StageId; 4] = [
    STAGE_VALIDATE_PRE,
    STAGE_STATS_NEUTRAL,
    STAGE_QC_POST,
    STAGE_SCREEN,
];
