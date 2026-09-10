//! Reconcile typed operational binding coverage without acquiring artifacts.
use crate::{
    AllDomainExpectedBenchmarkResultsReport, AllDomainOutputDeclarationStatus,
    AllDomainOutputDeclarationsReport, AllDomainParserCollectorReport,
    AllDomainParserCollectorSourceKind, AllDomainRenderedCommandsReport,
    AllDomainStageToolTableReport, OperationalBenchmarkReadyBlocker,
    OperationalBenchmarkReadyCheck, StageToolResourcesReport,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Default)]
pub struct OperationalBindingEvidence<'a> {
    pub expected_results: Option<&'a AllDomainExpectedBenchmarkResultsReport>,
    pub stage_tool_table: Option<&'a AllDomainStageToolTableReport>,
    pub rendered_commands: Option<&'a AllDomainRenderedCommandsReport>,
    pub output_declarations: Option<&'a AllDomainOutputDeclarationsReport>,
    pub parser_collector: Option<&'a AllDomainParserCollectorReport>,
    pub resource_report: Option<&'a StageToolResourcesReport>,
}

/// Reconcile coverage and retain every missing binding as an attributable blocker.
/// Parser coverage here refers to recorded local simulation evidence, not executed tool certification.
pub fn evaluate_operational_binding_coverage(
    evidence: OperationalBindingEvidence<'_>,
    blockers: &mut BTreeSet<OperationalBenchmarkReadyBlocker>,
    missing_expected_path: &str,
) -> OperationalBenchmarkReadyCheck {
    check_expected_bindings(evidence.expected_results, evidence.stage_tool_table, blockers);
    check_command_coverage(evidence.expected_results, evidence.rendered_commands, blockers);
    check_output_coverage(evidence.expected_results, evidence.output_declarations, blockers);
    check_parser_coverage(evidence.expected_results, evidence.parser_collector, blockers);
    check_resource_coverage(evidence.expected_results, evidence.resource_report, blockers);
    check_binding_metadata(evidence.expected_results, evidence.stage_tool_table, blockers);
    fastq_bam_binding_check(evidence, blockers, missing_expected_path)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BindingKey {
    domain: String,
    stage_id: String,
    tool_id: String,
    corpus_id: String,
    asset_profile_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PairKey {
    domain: String,
    stage_id: String,
    tool_id: String,
}

fn check_expected_bindings(
    expected_results: Option<&AllDomainExpectedBenchmarkResultsReport>,
    stage_tool_table: Option<&AllDomainStageToolTableReport>,
    blockers: &mut BTreeSet<OperationalBenchmarkReadyBlocker>,
) {
    if let (Some(expected), Some(table)) = (expected_results, stage_tool_table) {
        let expected_keys =
            expected.rows.iter().map(binding_key_from_expected).collect::<BTreeSet<_>>();
        for row in table.rows.iter().filter(|row| row.benchmark_status == "benchmark_ready") {
            let binding = binding_key_from_table(row);
            if !expected_keys.contains(&binding) {
                blockers.insert(binding_blocker(
                    &binding,
                    "expected_result_missing",
                    &expected.output_path,
                    "benchmark-ready binding is missing a canonical expected-result row",
                ));
            }
        }
    }
}

fn check_command_coverage(
    expected_results: Option<&AllDomainExpectedBenchmarkResultsReport>,
    rendered_commands: Option<&AllDomainRenderedCommandsReport>,
    blockers: &mut BTreeSet<OperationalBenchmarkReadyBlocker>,
) {
    if let (Some(expected), Some(rendered)) = (expected_results, rendered_commands) {
        let rendered_result_ids =
            rendered.rows.iter().map(|row| row.result_id.as_str()).collect::<BTreeSet<_>>();
        for row in &expected.rows {
            if !rendered_result_ids.contains(row.result_id.as_str()) {
                blockers.insert(binding_blocker(
                    &binding_key_from_expected(row),
                    "adapter_command_missing",
                    &rendered.output_path,
                    "canonical benchmark result is missing rendered command coverage",
                ));
            }
        }
    }
}

fn check_output_coverage(
    expected_results: Option<&AllDomainExpectedBenchmarkResultsReport>,
    output_declarations: Option<&AllDomainOutputDeclarationsReport>,
    blockers: &mut BTreeSet<OperationalBenchmarkReadyBlocker>,
) {
    if let (Some(expected), Some(outputs)) = (expected_results, output_declarations) {
        let output_by_result = outputs
            .rows
            .iter()
            .map(|row| (row.result_id.as_str(), row))
            .collect::<BTreeMap<_, _>>();
        for row in &expected.rows {
            match output_by_result.get(row.result_id.as_str()) {
                None => {
                    blockers.insert(binding_blocker(
                        &binding_key_from_expected(row),
                        "output_declaration_missing",
                        &outputs.output_path,
                        "canonical benchmark result is missing output declarations",
                    ));
                }
                Some(declaration)
                    if declaration.status != AllDomainOutputDeclarationStatus::Complete =>
                {
                    blockers.insert(binding_blocker(
                        &binding_key_from_expected(row),
                        "output_declaration_incomplete",
                        &outputs.output_path,
                        "canonical benchmark result has incomplete declared outputs",
                    ));
                }
                Some(_) => {}
            }
        }
    }
}

fn check_parser_coverage(
    expected_results: Option<&AllDomainExpectedBenchmarkResultsReport>,
    parser_collector: Option<&AllDomainParserCollectorReport>,
    blockers: &mut BTreeSet<OperationalBenchmarkReadyBlocker>,
) {
    if let (Some(expected), Some(parser_rows)) = (expected_results, parser_collector) {
        let fake_run_result_ids = parser_rows
            .rows
            .iter()
            .filter(|row| row.source_kind == AllDomainParserCollectorSourceKind::FakeRun)
            .filter_map(|row| row.result_id.as_deref())
            .collect::<BTreeSet<_>>();
        for row in &expected.rows {
            if !fake_run_result_ids.contains(row.result_id.as_str()) {
                blockers.insert(binding_blocker(
                    &binding_key_from_expected(row),
                    "parser_evidence_missing",
                    &parser_rows.output_path,
                    "canonical benchmark result is missing fake-run parser evidence",
                ));
            }
        }
    }
}

fn check_resource_coverage(
    expected_results: Option<&AllDomainExpectedBenchmarkResultsReport>,
    resource_report: Option<&StageToolResourcesReport>,
    blockers: &mut BTreeSet<OperationalBenchmarkReadyBlocker>,
) {
    if let (Some(expected), Some(resources)) = (expected_results, resource_report) {
        let resource_pairs =
            resources.rows.iter().map(binding_key_from_resource).collect::<BTreeSet<_>>();
        for row in &expected.rows {
            let binding = binding_key_from_expected(row);
            if !resource_pairs.contains(&pair_key_from_expected(row)) {
                blockers.insert(binding_blocker(
                    &binding,
                    "resource_hint_missing",
                    &resources.config_path,
                    "canonical benchmark result is missing governed resource hints",
                ));
            }
        }
    }
}

fn check_binding_metadata(
    expected_results: Option<&AllDomainExpectedBenchmarkResultsReport>,
    stage_tool_table: Option<&AllDomainStageToolTableReport>,
    blockers: &mut BTreeSet<OperationalBenchmarkReadyBlocker>,
) {
    if let (Some(expected), Some(table)) = (expected_results, stage_tool_table) {
        let table_by_binding = table
            .rows
            .iter()
            .filter(|row| row.benchmark_status == "benchmark_ready")
            .map(|row| (binding_key_from_table(row), row))
            .collect::<BTreeMap<_, _>>();
        for row in &expected.rows {
            let binding = binding_key_from_expected(row);
            match table_by_binding.get(&binding) {
                None => {
                    blockers.insert(binding_blocker(
                    &binding,
                    "stage_tool_binding_missing",
                    &table.output_path,
                    "canonical benchmark result is missing from the all-domain stage/tool table",
                ));
                }
                Some(table_row)
                    if table_row.corpus_id.trim().is_empty()
                        || table_row.asset_profile_id.trim().is_empty()
                        || table_row.adapter_id.trim().is_empty()
                        || table_row.parser_id.trim().is_empty() =>
                {
                    blockers.insert(binding_blocker(
                        &binding,
                        "binding_metadata_incomplete",
                        &table.output_path,
                        "stage/tool table binding is missing corpus, asset, adapter, or parser identity",
                    ));
                }
                Some(_) => {}
            }
        }
    }
}
fn fastq_bam_binding_check(
    evidence: OperationalBindingEvidence<'_>,
    blockers: &BTreeSet<OperationalBenchmarkReadyBlocker>,
    missing_expected_path: &str,
) -> OperationalBenchmarkReadyCheck {
    let expected_results = evidence.expected_results;
    match expected_results {
        Some(report) => {
            let binding_count = report
                .rows
                .iter()
                .filter(|row| row.domain == "fastq" || row.domain == "bam")
                .count();
            let blocker_count = blockers
                .iter()
                .filter(|blocker| blocker.domain == "fastq" || blocker.domain == "bam")
                .count();
            let mut missing_prerequisites = Vec::new();
            if evidence.stage_tool_table.is_none() {
                missing_prerequisites.push("all_domain_stage_tool_table");
            }
            if evidence.rendered_commands.is_none() {
                missing_prerequisites.push("all_domain_rendered_commands");
            }
            if evidence.output_declarations.is_none() {
                missing_prerequisites.push("all_domain_output_declarations");
            }
            if evidence.parser_collector.is_none() {
                missing_prerequisites.push("all_domain_parser_collector");
            }
            if evidence.resource_report.is_none() {
                missing_prerequisites.push("stage_tool_resources");
            }
            let ok = binding_count > 0 && blocker_count == 0 && missing_prerequisites.is_empty();
            let detail = if ok {
                format!(
                    "validated {binding_count} FASTQ/BAM benchmark-ready bindings through expected-result, command, parser, resource, and output coverage"
                )
            } else if !missing_prerequisites.is_empty() {
                format!(
                    "FASTQ/BAM benchmark binding coverage is missing prerequisite surfaces: {}",
                    missing_prerequisites.join(", ")
                )
            } else {
                format!(
                    "FASTQ/BAM benchmark binding coverage found {blocker_count} blockers across {binding_count} bindings"
                )
            };
            OperationalBenchmarkReadyCheck {
                surface_id: "fastq_bam_benchmark_binding_coverage".to_string(),
                output_path: report.output_path.clone(),
                ok,
                detail,
            }
        }
        None => OperationalBenchmarkReadyCheck {
            surface_id: "fastq_bam_benchmark_binding_coverage".to_string(),
            output_path: missing_expected_path.to_string(),
            ok: false,
            detail: "all-domain expected benchmark results did not render".to_string(),
        },
    }
}

fn binding_key_from_expected(row: &crate::AllDomainExpectedBenchmarkResultRow) -> BindingKey {
    BindingKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn binding_key_from_table(row: &crate::AllDomainStageToolTableRow) -> BindingKey {
    BindingKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn binding_key_from_resource(row: &crate::StageToolResourceRow) -> PairKey {
    PairKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
    }
}

fn pair_key_from_expected(row: &crate::AllDomainExpectedBenchmarkResultRow) -> PairKey {
    PairKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
    }
}

fn binding_blocker(
    binding: &BindingKey,
    blocker_type: &str,
    blocker_path: &str,
    detail: &str,
) -> OperationalBenchmarkReadyBlocker {
    OperationalBenchmarkReadyBlocker {
        domain: binding.domain.clone(),
        stage_id: binding.stage_id.clone(),
        tool_id: binding.tool_id.clone(),
        corpus_id: binding.corpus_id.clone(),
        asset_profile_id: binding.asset_profile_id.clone(),
        blocker_type: blocker_type.to_string(),
        blocker_path: blocker_path.to_string(),
        detail: detail.to_string(),
    }
}

#[cfg(test)]
#[path = "../../../tests/contracts/operational_binding_coverage.rs"]
mod tests;
