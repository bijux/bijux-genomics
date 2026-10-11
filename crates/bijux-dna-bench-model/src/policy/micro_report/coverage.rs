//! Owner: expected, duplicated, unavailable and missing micro execution coverage.
use crate::{
    MicroBenchmarkExecutionStatus, MicroBenchmarkFailedRow, MicroBenchmarkMissingRow,
    MicroBenchmarkReportSources, MicroBenchmarkResultRow,
};
use std::collections::{BTreeMap, BTreeSet};
pub(super) fn coverage(
    sources: MicroBenchmarkReportSources<'_>,
    existing_evidence_paths: &BTreeSet<String>,
) -> (Vec<MicroBenchmarkFailedRow>, Vec<MicroBenchmarkMissingRow>) {
    let result_rows = sources.results;
    let expected_rows = sources.expected;
    let actual_key_counts =
        result_rows.iter().fold(BTreeMap::<(&str, &str), usize>::new(), |mut counts, row| {
            *counts.entry((row.component_id.as_str(), row.execution_id.as_str())).or_default() += 1;
            counts
        });
    let expected_keys = expected_rows
        .iter()
        .map(|row| (row.component_id.as_str(), row.execution_id.as_str()))
        .collect::<BTreeSet<_>>();

    let mut failed_rows = Vec::new();
    for ((component_id, execution_id), count) in &actual_key_counts {
        if *count > 1 {
            failed_rows.push(MicroBenchmarkFailedRow {
                row_id: format!("duplicate:{component_id}:{execution_id}"),
                source_surface: "micro_result_rows".to_string(),
                component_id: Some((*component_id).to_string()),
                execution_id: Some((*execution_id).to_string()),
                domain: None,
                stage_id: None,
                tool_id: None,
                detail: format!(
                    "micro benchmark result rows duplicated `{component_id}` / `{execution_id}` {count} times"
                ),
            });
        }
    }
    result_failures(result_rows, &expected_keys, existing_evidence_paths, &mut failed_rows);
    artifact_failures(sources, &mut failed_rows);
    let missing_rows = expected_rows
        .iter()
        .filter(|row| {
            !actual_key_counts.contains_key(&(row.component_id.as_str(), row.execution_id.as_str()))
        })
        .map(|row| MicroBenchmarkMissingRow {
            component_id: row.component_id.clone(),
            execution_id: row.execution_id.clone(),
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            source_report_path: row.source_report_path.clone(),
            detail: "source-report row was not collected into the micro benchmark result set"
                .to_string(),
        })
        .collect::<Vec<_>>();

    (failed_rows, missing_rows)
}

fn result_failures(
    result_rows: &[MicroBenchmarkResultRow],
    expected_keys: &BTreeSet<(&str, &str)>,
    existing_evidence_paths: &BTreeSet<String>,
    failed_rows: &mut Vec<MicroBenchmarkFailedRow>,
) {
    for row in result_rows {
        if !expected_keys.contains(&(row.component_id.as_str(), row.execution_id.as_str())) {
            failed_rows.push(MicroBenchmarkFailedRow {
                row_id: format!("unexpected:{}:{}", row.component_id, row.execution_id),
                source_surface: "micro_result_rows".to_string(),
                component_id: Some(row.component_id.clone()),
                execution_id: Some(row.execution_id.clone()),
                domain: Some(row.domain.clone()),
                stage_id: Some(row.stage_id.clone()),
                tool_id: Some(row.tool_id.clone()),
                detail: "micro benchmark result row has no matching source-report row".to_string(),
            });
        }
        if row.status == MicroBenchmarkExecutionStatus::Succeeded {
            match &row.evidence_path {
                Some(evidence_path) if existing_evidence_paths.contains(evidence_path) => {}
                Some(_) => failed_rows.push(MicroBenchmarkFailedRow {
                    row_id: format!("missing-evidence:{}:{}", row.component_id, row.execution_id),
                    source_surface: "micro_result_rows".to_string(),
                    component_id: Some(row.component_id.clone()),
                    execution_id: Some(row.execution_id.clone()),
                    domain: Some(row.domain.clone()),
                    stage_id: Some(row.stage_id.clone()),
                    tool_id: Some(row.tool_id.clone()),
                    detail: "successful micro benchmark row references a missing evidence file"
                        .to_string(),
                }),
                None => failed_rows.push(MicroBenchmarkFailedRow {
                    row_id: format!("blank-evidence:{}:{}", row.component_id, row.execution_id),
                    source_surface: "micro_result_rows".to_string(),
                    component_id: Some(row.component_id.clone()),
                    execution_id: Some(row.execution_id.clone()),
                    domain: Some(row.domain.clone()),
                    stage_id: Some(row.stage_id.clone()),
                    tool_id: Some(row.tool_id.clone()),
                    detail: "successful micro benchmark row is missing an evidence path"
                        .to_string(),
                }),
            }
        }
    }
}

fn artifact_failures(
    sources: MicroBenchmarkReportSources<'_>,
    failed_rows: &mut Vec<MicroBenchmarkFailedRow>,
) {
    let output_rows = sources.outputs;
    let log_rows = sources.logs;
    for output_row in output_rows.iter().filter(|row| !row.exists) {
        failed_rows.push(MicroBenchmarkFailedRow {
            row_id: format!(
                "missing-output:{}:{}",
                output_row.component_id, output_row.execution_id
            ),
            source_surface: "micro_output_rows".to_string(),
            component_id: Some(output_row.component_id.clone()),
            execution_id: Some(output_row.execution_id.clone()),
            domain: None,
            stage_id: None,
            tool_id: None,
            detail: format!("output row references a missing path `{}`", output_row.path),
        });
    }
    for log_row in log_rows.iter().filter(|row| !row.exists) {
        failed_rows.push(MicroBenchmarkFailedRow {
            row_id: format!("missing-log:{}:{}", log_row.component_id, log_row.execution_id),
            source_surface: "micro_log_rows".to_string(),
            component_id: Some(log_row.component_id.clone()),
            execution_id: Some(log_row.execution_id.clone()),
            domain: None,
            stage_id: None,
            tool_id: None,
            detail: format!("log row references a missing path `{}`", log_row.path),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MicroBenchmarkResultKind;
    #[test]
    fn succeeded_evidence_and_upstream_coverage_are_independent_obligations() {
        let mut row = MicroBenchmarkResultRow {
            execution_id: "reads".into(),
            component_id: "fastq".into(),
            domain: "fastq".into(),
            result_kind: MicroBenchmarkResultKind::Stage,
            bridge_source_domain: None,
            bridge_target_domain: None,
            stage_id: "fastq.validate_reads".into(),
            tool_id: "fastqvalidator".into(),
            status: MicroBenchmarkExecutionStatus::Succeeded,
            reason: "observed".into(),
            command: Some("validate".into()),
            source_report_path: "source.json".into(),
            evidence_path: Some("evidence.json".into()),
            stage_result_manifest_path: None,
            normalized_metric_count: 0,
            output_count: 1,
            log_count: 0,
        };
        let known = BTreeSet::from([("fastq", "reads")]);
        let present = BTreeSet::from(["evidence.json".into()]);
        let mut failures = Vec::new();
        result_failures(std::slice::from_ref(&row), &known, &present, &mut failures);
        assert!(failures.is_empty());
        result_failures(
            std::slice::from_ref(&row),
            &BTreeSet::new(),
            &BTreeSet::new(),
            &mut failures,
        );
        assert_eq!(
            failures.iter().map(|r| r.row_id.as_str()).collect::<Vec<_>>(),
            ["unexpected:fastq:reads", "missing-evidence:fastq:reads"]
        );
        failures.clear();
        row.evidence_path = None;
        result_failures(std::slice::from_ref(&row), &known, &present, &mut failures);
        assert_eq!(failures[0].row_id, "blank-evidence:fastq:reads");
        failures.clear();
        row.status = MicroBenchmarkExecutionStatus::Unavailable;
        result_failures(std::slice::from_ref(&row), &known, &BTreeSet::new(), &mut failures);
        assert!(failures.is_empty());
    }
}
