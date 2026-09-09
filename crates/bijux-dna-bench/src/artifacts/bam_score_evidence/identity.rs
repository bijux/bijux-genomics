//! Owner: bijux-dna-bench
//! Interpret observed BAM identity report fields without filesystem access.

use super::fields::{
    format_f64, optional_str, required_array, required_bool, required_f64, required_str,
    required_u64, summarize_metrics,
};
use super::merge::insert_evidence;
use super::{BamEvidenceAggregate, EvidenceRows};
use anyhow::Result;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(crate) fn merge_contamination_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    let stage_id = required_str(root, "stage_id")?;
    let by_tool = required_array(root, "rows")?.iter().fold(
        BTreeMap::<String, Vec<&Value>>::new(),
        |mut map, row| {
            if let Some(tool_id) = optional_str(row, "tool_id") {
                map.entry(tool_id.to_string()).or_default().push(row);
            }
            map
        },
    );
    for (tool_id, tool_rows) in by_tool {
        let selected =
            tool_rows.iter().find(|row| optional_str(row, "proof_case") == Some("ready")).copied();
        let Some(row) = selected else {
            continue;
        };
        let raw_estimate = required_f64(row, "raw_estimate")?;
        let raw_ci_low = required_f64(row, "raw_ci_low")?;
        let raw_ci_high = required_f64(row, "raw_ci_high")?;
        insert_evidence(
            rows,
            stage_id,
            &tool_id,
            BamEvidenceAggregate {
                truth_correctness_score: Some((1.0 - raw_estimate).clamp(0.0, 1.0)),
                truth_correctness_basis: Some("one_minus_contamination_estimate".to_string()),
                contract_correctness_score: Some(if required_bool(row, "expectation_matched")?
                    && required_bool(row, "prerequisites_passed")?
                {
                    1.0
                } else {
                    0.0
                }),
                contract_correctness_basis: Some(
                    "contamination smoke report preserved governed estimate and prerequisite surfaces"
                        .to_string(),
                ),
                authenticity_metric_value: Some(raw_estimate),
                authenticity_metric_basis: Some("estimate".to_string()),
                scientific_metric_summary: summarize_metrics(vec![
                    format!("estimate={}", format_f64(raw_estimate)),
                    format!("ci_low={}", format_f64(raw_ci_low)),
                    format!("ci_high={}", format_f64(raw_ci_high)),
                ]),
                source_paths: BTreeSet::new(),
                ..BamEvidenceAggregate::default()
            },
            path,
        );
    }
    Ok(())
}

pub(crate) fn merge_sex_summary(root: &Value, path: &Path, rows: &mut EvidenceRows) -> Result<()> {
    let stage_id = required_str(root, "stage_id")?;
    let by_tool = required_array(root, "rows")?.iter().fold(
        BTreeMap::<String, Vec<&Value>>::new(),
        |mut map, row| {
            if let Some(tool_id) = optional_str(row, "tool_id") {
                map.entry(tool_id.to_string()).or_default().push(row);
            }
            map
        },
    );
    for (tool_id, tool_rows) in by_tool {
        let selected =
            tool_rows.iter().find(|row| optional_str(row, "proof_case") == Some("ready")).copied();
        let Some(row) = selected else {
            continue;
        };
        let confidence = required_f64(row, "confidence")?;
        insert_evidence(
            rows,
            stage_id,
            &tool_id,
            BamEvidenceAggregate {
                truth_correctness_score: Some(confidence),
                truth_correctness_basis: Some("confidence".to_string()),
                contract_correctness_score: Some(if required_bool(row, "expectation_matched")? {
                    1.0
                } else {
                    0.0
                }),
                contract_correctness_basis: Some(
                    "sex tool smoke report preserved governed call, confidence, and coverage surfaces"
                        .to_string(),
                ),
                scientific_metric_summary: summarize_metrics(vec![
                    format!("call={}", required_str(row, "call")?),
                    format!("status={}", required_str(row, "status")?),
                    format!("x_coverage={}", format_f64(required_f64(row, "x_coverage")?)),
                    format!("y_coverage={}", format_f64(required_f64(row, "y_coverage")?)),
                    format!(
                        "autosomal_coverage={}",
                        format_f64(required_f64(row, "autosomal_coverage")?)
                    ),
                ]),
                source_paths: BTreeSet::new(),
                ..BamEvidenceAggregate::default()
            },
            path,
        );
    }
    Ok(())
}

pub(crate) fn merge_haplogroups_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    let stage_id = required_str(root, "stage_id")?;
    let ready_row = required_array(root, "rows")?
        .iter()
        .find(|row| optional_str(row, "proof_case") == Some("ready"));
    let Some(row) = ready_row else {
        return Ok(());
    };
    insert_evidence(
        rows,
        stage_id,
        required_str(row, "tool_id")?,
        BamEvidenceAggregate {
            contract_correctness_score: Some(if required_bool(row, "expectation_matched")? {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "haplogroup smoke report preserved governed call and marker support surfaces"
                    .to_string(),
            ),
            authenticity_metric_value: Some(required_f64(row, "confidence")?),
            authenticity_metric_basis: Some("confidence".to_string()),
            scientific_metric_summary: summarize_metrics(vec![
                format!("haplogroup_call={}", optional_str(row, "haplogroup_call").unwrap_or("")),
                format!("markers_supported={}", required_u64(row, "markers_supported")?),
                format!("markers_total={}", required_u64(row, "markers_total")?),
                format!("status={}", required_str(row, "status")?),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}

pub(crate) fn merge_kinship_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    let stage_id = required_str(root, "stage_id")?;
    let case = required_array(root, "cases")?.iter().find(|case| {
        optional_str(case, "method") == Some("king") && optional_str(case, "status") == Some("ok")
    });
    let Some(case) = case else {
        return Ok(());
    };
    let concordance = case
        .get("pairwise_results")
        .and_then(Value::as_array)
        .and_then(|rows| rows.first())
        .and_then(|row| row.get("concordance"))
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    insert_evidence(
        rows,
        stage_id,
        "king",
        BamEvidenceAggregate {
            truth_correctness_score: Some(concordance),
            truth_correctness_basis: Some("max_pairwise_concordance".to_string()),
            contract_correctness_score: Some(if required_bool(case, "expectation_matched")? {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "kinship smoke report preserved governed pairwise relationship outputs".to_string(),
            ),
            scientific_metric_summary: summarize_metrics(vec![
                format!("pair_count={}", required_u64(case, "pair_count")?),
                format!(
                    "observed_max_overlap_snps={}",
                    required_u64(case, "observed_max_overlap_snps")?
                ),
                format!("status={}", required_str(case, "status")?),
                format!("concordance={}", format_f64(concordance)),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}
