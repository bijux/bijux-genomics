//! Owner: bijux-dna-bench
//! Interpret observed BAM damage report fields without filesystem access.

use super::fields::{format_f64, required_bool, required_f64, required_str, summarize_metrics};
use super::merge::insert_evidence;
use super::{BamEvidenceAggregate, EvidenceRows};
use anyhow::Result;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::Path;

pub(crate) fn merge_damage_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    let terminal_c_to_t_5p = required_f64(root, "terminal_c_to_t_5p")?;
    let terminal_g_to_a_3p = required_f64(root, "terminal_g_to_a_3p")?;
    let damage_signal = required_str(root, "damage_signal")?;
    let score = f64::midpoint(terminal_c_to_t_5p, terminal_g_to_a_3p);
    insert_evidence(
        rows,
        required_str(root, "stage_id")?,
        required_str(root, "method")?,
        BamEvidenceAggregate {
            truth_correctness_score: Some(score),
            truth_correctness_basis: Some("mean_terminal_substitution_rate".to_string()),
            contract_correctness_score: Some(if required_bool(root, "expectation_matched")? {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "damage smoke report preserved governed terminal substitution and damage label surfaces"
                    .to_string(),
            ),
            damage_metric_value: Some(score),
            damage_metric_basis: Some("mean_terminal_substitution_rate".to_string()),
            scientific_metric_summary: summarize_metrics(vec![
                format!("damage_signal={damage_signal}"),
                format!("terminal_c_to_t_5p={}", format_f64(terminal_c_to_t_5p)),
                format!("terminal_g_to_a_3p={}", format_f64(terminal_g_to_a_3p)),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}

pub(crate) fn merge_authenticity_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    let score = required_f64(root, "score")?;
    let confidence = required_f64(root, "confidence")?;
    let pmd_like_signal_present = required_bool(root, "pmd_like_signal_present")?;
    let status = required_str(root, "status")?;
    let truth_score = (score
        + confidence
        + f64::from(u8::from(pmd_like_signal_present))
        + if status == "pass" { 1.0 } else { 0.0 })
        / 4.0;
    insert_evidence(
        rows,
        required_str(root, "stage_id")?,
        required_str(root, "method")?,
        BamEvidenceAggregate {
            truth_correctness_score: Some(truth_score),
            truth_correctness_basis: Some("authenticity_metric_mean".to_string()),
            contract_correctness_score: Some(if required_bool(root, "expectation_matched")? {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "authenticity smoke report preserved governed score, confidence, and status surfaces"
                    .to_string(),
            ),
            authenticity_metric_value: Some(score),
            authenticity_metric_basis: Some("score".to_string()),
            scientific_metric_summary: summarize_metrics(vec![
                format!("confidence={}", format_f64(confidence)),
                format!("pmd_like_signal_present={pmd_like_signal_present}"),
                format!("status={status}"),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}
