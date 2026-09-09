//! Owner: bijux-dna-bench-model
//! BAM evidence selection and weighted scoring without repository IO.

use crate::{
    BamToolScoreRow, BamToolScoringInput, StageScoringCorrectnessSignal, StageScoringDecisionMode,
};

mod assembly;
mod ranking;
mod report;
mod selection;
use ranking::{
    build_memory_scores, build_runtime_scores, completion_score_for_row, correctness_signal_label,
    decision_mode_label, score_row,
};
pub use report::build_bam_tool_scores_report;

#[derive(Debug, Clone)]
struct BaseScoreRow {
    stage_id: String,
    tool_id: String,
    decision_mode: StageScoringDecisionMode,
    correctness_signal: StageScoringCorrectnessSignal,
    weights: crate::StageScoringWeights,
    scientific_metric_ids: Vec<String>,
    result_ids: Vec<String>,
    report_row_ids: Vec<String>,
    corpus_ids: Vec<String>,
    report_sections: Vec<String>,
    row_statuses: Vec<String>,
    truth_correctness_score: Option<f64>,
    truth_correctness_basis: Option<String>,
    contract_correctness_score: Option<f64>,
    contract_correctness_basis: Option<String>,
    retained_reads: Option<u64>,
    dropped_reads: Option<u64>,
    alignment_qc_metric_value: Option<f64>,
    alignment_qc_metric_basis: Option<String>,
    coverage_metric_value: Option<f64>,
    coverage_metric_basis: Option<String>,
    damage_metric_value: Option<f64>,
    damage_metric_basis: Option<String>,
    authenticity_metric_value: Option<f64>,
    authenticity_metric_basis: Option<String>,
    scientific_metric_summary: Option<String>,
    runtime_seconds: Option<f64>,
    runtime_source: String,
    micro_execution_status: Option<String>,
    effective_memory_mb: Option<f64>,
    observed_memory_mb: Option<f64>,
    declared_memory_mb: Option<f64>,
    memory_source: String,
    failure_class: String,
    evidence_paths: Vec<String>,
    reason: String,
}

/// Score typed BAM observations while preserving missing and blocked evidence states.
///
/// # Errors
/// Rejects a scoring configuration that violates the shared policy contract.
pub fn build_bam_tool_score_rows(
    input: BamToolScoringInput,
) -> anyhow::Result<Vec<BamToolScoreRow>> {
    crate::validate_stage_scoring_config(&input.config)?;
    Ok(score_base_rows(assembly::collect_base_score_rows(input)))
}

fn score_base_rows(base_rows: Vec<BaseScoreRow>) -> Vec<BamToolScoreRow> {
    let runtime_scores = build_runtime_scores(&base_rows);
    let memory_scores = build_memory_scores(&base_rows);
    let mut rows = Vec::new();
    for row in base_rows {
        let correctness_component = match row.correctness_signal {
            StageScoringCorrectnessSignal::ScientificComparableMetrics => {
                row.truth_correctness_score
            }
            StageScoringCorrectnessSignal::OutputContract => row.contract_correctness_score,
        };
        let scientific_threshold_score = if row.correctness_signal
            == StageScoringCorrectnessSignal::ScientificComparableMetrics
        {
            row.truth_correctness_score
        } else {
            None
        };
        let completion_score = completion_score_for_row(&row);
        let failure_class_score = Some(if row.failure_class == "none" { 1.0 } else { 0.0 });
        let runtime_score =
            runtime_scores.get(&(row.stage_id.clone(), row.tool_id.clone())).copied();
        let memory_score = memory_scores.get(&(row.stage_id.clone(), row.tool_id.clone())).copied();
        let (score_status, score_weight_coverage, score_total) = score_row(
            &row,
            correctness_component,
            scientific_threshold_score,
            runtime_score,
            memory_score,
            completion_score,
            failure_class_score,
        );

        rows.push(BamToolScoreRow {
            stage_id: row.stage_id,
            tool_id: row.tool_id,
            decision_mode: decision_mode_label(row.decision_mode).to_string(),
            correctness_signal: correctness_signal_label(row.correctness_signal).to_string(),
            result_ids: row.result_ids,
            report_row_ids: row.report_row_ids,
            corpus_ids: row.corpus_ids,
            report_sections: row.report_sections,
            row_statuses: row.row_statuses,
            score_status,
            truth_correctness_score: row.truth_correctness_score,
            truth_correctness_basis: row.truth_correctness_basis,
            contract_correctness_score: row.contract_correctness_score,
            contract_correctness_basis: row.contract_correctness_basis,
            retained_reads: row.retained_reads,
            dropped_reads: row.dropped_reads,
            alignment_qc_metric_value: row.alignment_qc_metric_value,
            alignment_qc_metric_basis: row.alignment_qc_metric_basis,
            coverage_metric_value: row.coverage_metric_value,
            coverage_metric_basis: row.coverage_metric_basis,
            damage_metric_value: row.damage_metric_value,
            damage_metric_basis: row.damage_metric_basis,
            authenticity_metric_value: row.authenticity_metric_value,
            authenticity_metric_basis: row.authenticity_metric_basis,
            scientific_metric_ids: row.scientific_metric_ids,
            scientific_metric_summary: row.scientific_metric_summary,
            runtime_seconds: row.runtime_seconds,
            runtime_source: row.runtime_source,
            observed_memory_mb: row.observed_memory_mb,
            declared_memory_mb: row.declared_memory_mb,
            memory_source: row.memory_source,
            failure_class: row.failure_class,
            micro_execution_status: row.micro_execution_status,
            score_weight_coverage,
            score_total,
            evidence_paths: row.evidence_paths,
            reason: row.reason,
        });
    }

    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StageScoringWeights;
    use crate::{
        BamScoreEvidence, BamScoreExpectedRow, BamScoreMemoryRow, BamScoreRuntimeRow,
        BamToolScoreStatus, StageScoringApplicability, StageScoringConfig,
    };
    use anyhow::Result;
    use std::collections::{BTreeMap, BTreeSet};

    fn input() -> Result<BamToolScoringInput> {
        let mut config: StageScoringConfig = serde_json::from_str(include_str!(
            "../../../tests/fixtures/stage_scoring/policy.json"
        ))?;
        let row = &mut config.rows[0];
        row.domain = "bam".into();
        row.stage_id = "bam.validate_alignment".into();
        row.default_tool_id = "alpha".into();
        row.active_tool_count = 2;
        row.benchmark_ready_tool_count = 2;
        row.benchmark_ready_tool_ids = vec!["alpha".into(), "beta".into()];
        row.correctness.signal = StageScoringCorrectnessSignal::OutputContract;
        row.scientific_threshold.applicability = StageScoringApplicability::NotApplicable;
        row.scientific_threshold.metric_ids.clear();
        row.weights = StageScoringWeights {
            correctness: 0.7,
            scientific_threshold: 0.0,
            runtime: 0.1,
            memory: 0.1,
            completion: 0.1,
            failure_class: 0.0,
        };
        let mut input = BamToolScoringInput {
            config,
            expected_rows: Vec::new(),
            runtime_rows: Vec::new(),
            memory_rows: Vec::new(),
            evidence: BTreeMap::new(),
            external_artifact_bindings: BTreeSet::new(),
        };
        for (tool, correctness, elapsed, memory) in
            [("alpha", 1.0, 1.0, 10.0), ("beta", 0.5, 3.0, 20.0)]
        {
            let stage = "bam.validate_alignment";
            input.expected_rows.push(BamScoreExpectedRow {
                domain: "bam".into(),
                stage_id: stage.into(),
                tool_id: tool.into(),
                result_id: Some(format!("result-{tool}")),
                report_row_id: format!("report-{tool}"),
                corpus_id: "synthetic".into(),
                report_section: "validation".into(),
                row_status: "present".into(),
            });
            input.runtime_rows.push(BamScoreRuntimeRow {
                domain: "bam".into(),
                stage_id: stage.into(),
                tool_id: tool.into(),
                execution_status: "succeeded".into(),
                elapsed_seconds: Some(elapsed),
                runtime_source: "synthetic".into(),
            });
            input.memory_rows.push(BamScoreMemoryRow {
                domain: "bam".into(),
                stage_id: stage.into(),
                tool_id: tool.into(),
                execution_status: "succeeded".into(),
                observed_memory_mb: Some(memory),
                declared_memory_mb: Some(100.0),
                memory_source: "synthetic".into(),
            });
            input.evidence.insert(
                (stage.into(), tool.into()),
                BamScoreEvidence {
                    contract_correctness_score: Some(correctness),
                    contract_correctness_basis: Some("synthetic unit evidence".into()),
                    ..BamScoreEvidence::default()
                },
            );
        }
        Ok(input)
    }

    #[test]
    fn bam_scoring_preserves_weighted_ranking_and_input_order_independence() -> Result<()> {
        let original = input()?;
        let first = build_bam_tool_score_rows(original.clone())?;
        assert_eq!(first.len(), 2);
        assert_eq!(first[0].score_status, BamToolScoreStatus::Scored);
        assert!((first[0].score_total.unwrap_or(-1.0) - 1.0).abs() < 1e-12);
        assert!((first[1].score_total.unwrap_or(-1.0) - 0.45).abs() < 1e-12);
        let mut reversed = original;
        reversed.expected_rows.reverse();
        reversed.runtime_rows.reverse();
        reversed.memory_rows.reverse();
        reversed.config.rows[0].benchmark_ready_tool_ids.reverse();
        assert_eq!(
            serde_json::to_value(first)?,
            serde_json::to_value(build_bam_tool_score_rows(reversed)?)?
        );
        Ok(())
    }

    #[test]
    fn bam_scoring_retains_insufficient_and_blocked_evidence_states() -> Result<()> {
        let mut input = input()?;
        input.evidence.remove(&("bam.validate_alignment".into(), "alpha".into()));
        input.external_artifact_bindings.insert(("bam.validate_alignment".into(), "beta".into()));
        let rows = build_bam_tool_score_rows(input)?;
        assert_eq!(rows[0].score_status, BamToolScoreStatus::InsufficientEvidence);
        assert_eq!(rows[1].score_status, BamToolScoreStatus::Blocked);
        assert!(rows.iter().all(|row| row.score_total.is_none()));
        let report = build_bam_tool_scores_report("scores.tsv".into(), "policy.toml".into(), rows);
        assert_eq!(
            (
                report.scored_row_count,
                report.insufficient_evidence_row_count,
                report.blocked_row_count
            ),
            (0, 1, 1)
        );
        Ok(())
    }

    #[test]
    fn bam_scoring_requires_valid_policy_and_preserves_missing_metric_coverage() -> Result<()> {
        let mut missing = input()?;
        missing.runtime_rows.clear();
        missing.memory_rows.clear();
        let rows = build_bam_tool_score_rows(missing)?;
        assert!(rows
            .iter()
            .all(|row| row.runtime_seconds.is_none() && row.observed_memory_mb.is_none()));
        assert!(rows.iter().all(|row| (row.score_weight_coverage - 0.8).abs() < 1e-12));
        let mut invalid = input()?;
        invalid.config.rows[0].weights.runtime = f64::NAN;
        assert!(build_bam_tool_score_rows(invalid).is_err());
        Ok(())
    }
    #[test]
    fn bam_scoring_never_attributes_another_tools_metrics_to_the_default_binding() -> Result<()> {
        for observed_tool in ["beta", "unregistered"] {
            let mut input = input()?;
            input.runtime_rows.retain(|row| row.tool_id == "beta");
            input.memory_rows.retain(|row| row.tool_id == "beta");
            input.runtime_rows[0].tool_id = observed_tool.into();
            input.memory_rows[0].tool_id = observed_tool.into();
            let rows = build_bam_tool_score_rows(input)?;
            assert_eq!(rows[0].tool_id, "alpha");
            assert!(rows[0].runtime_seconds.is_none(), "foreign runtime from {observed_tool}");
            assert!(rows[0].observed_memory_mb.is_none(), "foreign memory from {observed_tool}");
            assert!(rows[0].declared_memory_mb.is_none());
            assert_eq!(rows[0].runtime_source, "not_available");
            assert_eq!(rows[0].memory_source, "not_available");
            if observed_tool == "beta" {
                assert_eq!(rows[1].runtime_seconds, Some(3.0));
                assert_eq!(rows[1].observed_memory_mb, Some(20.0));
            }
        }
        Ok(())
    }

    #[test]
    fn bam_scoring_keeps_execution_failure_at_its_observed_tool_binding() -> Result<()> {
        let mut input = input()?;
        input.runtime_rows.retain(|row| row.tool_id == "beta");
        input.runtime_rows[0].execution_status = "failed".into();
        let rows = build_bam_tool_score_rows(input)?;
        assert_eq!(rows[0].score_status, BamToolScoreStatus::Scored);
        assert_eq!(rows[0].failure_class, "none");
        assert_eq!(rows[1].score_status, BamToolScoreStatus::Blocked);
        assert_eq!(rows[1].failure_class, "command_failed");
        Ok(())
    }
}
