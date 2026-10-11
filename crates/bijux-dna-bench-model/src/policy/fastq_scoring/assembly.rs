//! Index typed input evidence and assemble one admitted stage/tool row at a time.

use super::selection::{
    classify_failure_class, describe_row_reason, select_memory_row, select_runtime_row,
};
use super::BaseScoreRow;
use crate::{
    FastqScoreEvidence, FastqScoreExpectedRow, FastqScoreMemoryRow, FastqScoreRuntimeRow,
    FastqToolScoringInput, StageScoringRow,
};
use std::collections::{BTreeMap, BTreeSet};

struct EvidenceIndex {
    full_rows_by_binding: BTreeMap<(String, String), Vec<FastqScoreExpectedRow>>,
    runtime_by_binding: BTreeMap<(String, String), Vec<FastqScoreRuntimeRow>>,
    memory_by_binding: BTreeMap<(String, String), Vec<FastqScoreMemoryRow>>,
    evidence: BTreeMap<(String, String), FastqScoreEvidence>,
    external_artifact_bindings: BTreeSet<(String, String)>,
    failure_catalog: BTreeMap<String, String>,
}

pub(super) fn collect_base_score_rows(input: FastqToolScoringInput) -> Vec<BaseScoreRow> {
    let FastqToolScoringInput {
        config,
        expected_rows,
        runtime_rows,
        memory_rows,
        evidence,
        external_artifact_bindings,
    } = input;
    let failure_catalog = config
        .failure_classes
        .iter()
        .map(|row| (row.class_id.clone(), row.detail.clone()))
        .collect::<BTreeMap<_, _>>();
    let full_rows_by_binding = expected_rows.into_iter().filter(|row| row.domain == "fastq").fold(
        BTreeMap::<(String, String), Vec<FastqScoreExpectedRow>>::new(),
        |mut map, row| {
            map.entry((row.stage_id.clone(), row.tool_id.clone())).or_default().push(row);
            map
        },
    );
    let runtime_by_binding = runtime_rows.into_iter().filter(|row| row.domain == "fastq").fold(
        BTreeMap::<(String, String), Vec<FastqScoreRuntimeRow>>::new(),
        |mut map, row| {
            map.entry((row.stage_id.clone(), row.tool_id.clone())).or_default().push(row);
            map
        },
    );
    let memory_by_binding = memory_rows.into_iter().filter(|row| row.domain == "fastq").fold(
        BTreeMap::<(String, String), Vec<FastqScoreMemoryRow>>::new(),
        |mut map, row| {
            map.entry((row.stage_id.clone(), row.tool_id.clone())).or_default().push(row);
            map
        },
    );

    let index = EvidenceIndex {
        full_rows_by_binding,
        runtime_by_binding,
        memory_by_binding,
        evidence,
        external_artifact_bindings,
        failure_catalog,
    };
    let mut rows = Vec::new();
    for row in config.rows.iter().filter(|row| row.domain == "fastq") {
        for tool in &row.benchmark_ready_tool_ids {
            rows.push(build_base_row(row, tool, &index));
        }
    }
    rows
}

fn build_base_row(
    stage_row: &StageScoringRow,
    tool_id: &str,
    index: &EvidenceIndex,
) -> BaseScoreRow {
    let binding_key = (stage_row.stage_id.clone(), tool_id.to_string());
    let binding_rows = index.full_rows_by_binding.get(&binding_key).cloned().unwrap_or_default();
    let evidence_row = index.evidence.get(&binding_key);
    let runtime_row = select_runtime_row(&stage_row.stage_id, tool_id, &index.runtime_by_binding);
    let memory_row = select_memory_row(&stage_row.stage_id, tool_id, &index.memory_by_binding);
    let failure_class = classify_failure_class(
        &binding_rows,
        evidence_row,
        runtime_row.as_ref(),
        index.external_artifact_bindings.contains(&binding_key),
    );
    let reason = describe_row_reason(
        &failure_class,
        evidence_row,
        runtime_row.as_ref(),
        memory_row.as_ref(),
        stage_row,
        tool_id,
        &index.failure_catalog,
    );

    BaseScoreRow {
        stage_id: stage_row.stage_id.clone(),
        tool_id: tool_id.to_string(),
        decision_mode: stage_row.decision_mode,
        correctness_signal: stage_row.correctness.signal,
        weights: stage_row.weights.clone(),
        result_ids: unique_strings(binding_rows.iter().filter_map(|row| row.result_id.clone())),
        report_row_ids: unique_strings(binding_rows.iter().map(|row| row.report_row_id.clone())),
        corpus_ids: unique_strings(binding_rows.iter().map(|row| row.corpus_id.clone())),
        report_sections: unique_strings(binding_rows.iter().map(|row| row.report_section.clone())),
        row_statuses: unique_strings(binding_rows.iter().map(|row| row.row_status.clone())),
        truth_correctness_score: evidence_row.and_then(|row| row.truth_correctness_score),
        truth_correctness_basis: evidence_row.and_then(|row| row.truth_correctness_basis.clone()),
        contract_correctness_score: evidence_row.and_then(|row| row.contract_correctness_score),
        contract_correctness_basis: evidence_row
            .and_then(|row| row.contract_correctness_basis.clone()),
        retained_reads: evidence_row.and_then(|row| row.retained_reads),
        dropped_reads: evidence_row.and_then(|row| row.dropped_reads),
        runtime_seconds: runtime_row.as_ref().and_then(|row| row.elapsed_seconds),
        runtime_source: runtime_row
            .as_ref()
            .map_or_else(|| "not_available".to_string(), |row| row.runtime_source.clone()),
        micro_execution_status: runtime_row
            .as_ref()
            .map(|row| row.execution_status.clone())
            .or_else(|| memory_row.as_ref().map(|row| row.execution_status.clone())),
        effective_memory_mb: memory_row
            .as_ref()
            .and_then(|row| row.observed_memory_mb.or(row.declared_memory_mb)),
        observed_memory_mb: memory_row.as_ref().and_then(|row| row.observed_memory_mb),
        declared_memory_mb: memory_row.as_ref().and_then(|row| row.declared_memory_mb),
        memory_source: memory_row
            .as_ref()
            .map_or_else(|| "not_available".to_string(), |row| row.memory_source.clone()),
        failure_class,
        evidence_paths: evidence_row
            .map(|row| row.source_paths.iter().cloned().collect::<Vec<_>>())
            .unwrap_or_default(),
        reason,
    }
}

fn unique_strings<I>(values: I) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    let mut seen = BTreeSet::new();
    let mut rows = Vec::new();
    for value in values {
        if seen.insert(value.clone()) {
            rows.push(value);
        }
    }
    rows
}
