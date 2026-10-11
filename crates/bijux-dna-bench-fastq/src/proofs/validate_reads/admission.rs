use super::binding::binding_key;
use super::contracts::ensure_required_tool_set;
use super::declarations::FASTQ_VALIDATE_READS_STAGE_ID;
use super::source_inputs::AdmittedBindings;
use super::source_inputs::SourceEvidence;
use anyhow::anyhow;
use anyhow::Result;
use std::collections::BTreeMap;
pub(super) fn admit(source: SourceEvidence) -> Result<AdmittedBindings> {
    let SourceEvidence {
        active_matrix,
        output_contract,
        parser_coverage,
        comparable_metrics,
        report_map,
        expected_results,
        command_argv,
        smoke,
    } = source;
    let active_rows = active_matrix
        .into_iter()
        .filter(|row| row.stage_id == FASTQ_VALIDATE_READS_STAGE_ID)
        .collect::<Vec<_>>();
    ensure_required_tool_set(
        "fastq validate-reads active rows",
        active_rows.iter().map(|row| row.tool_id.as_str()),
    )?;

    let output_rows = output_contract
        .into_iter()
        .filter(|row| row.stage_id == FASTQ_VALIDATE_READS_STAGE_ID)
        .collect::<Vec<_>>();
    ensure_required_tool_set(
        "fastq validate-reads output-contract rows",
        output_rows.iter().map(|row| row.tool_id.as_str()),
    )?;

    let parser_rows = parser_coverage
        .into_iter()
        .filter(|row| row.stage_id == FASTQ_VALIDATE_READS_STAGE_ID)
        .collect::<Vec<_>>();
    ensure_required_tool_set(
        "fastq validate-reads parser rows",
        parser_rows.iter().map(|row| row.tool_id.as_str()),
    )?;

    let expected_rows = expected_results
        .into_iter()
        .filter(|row| row.domain == "fastq" && row.stage_id == FASTQ_VALIDATE_READS_STAGE_ID)
        .collect::<Vec<_>>();
    ensure_required_tool_set(
        "fastq validate-reads expected-result rows",
        expected_rows.iter().map(|row| row.tool_id.as_str()),
    )?;

    let command_rows = command_argv
        .into_iter()
        .filter(|row| row.stage_id == FASTQ_VALIDATE_READS_STAGE_ID)
        .collect::<Vec<_>>();
    ensure_required_tool_set(
        "fastq validate-reads rendered-command rows",
        command_rows.iter().map(|row| row.tool_id.as_str()),
    )?;

    let comparable_row = comparable_metrics
        .into_iter()
        .find(|row| row.stage_id == FASTQ_VALIDATE_READS_STAGE_ID)
        .ok_or_else(|| {
            anyhow!("missing comparable-metrics row for `{FASTQ_VALIDATE_READS_STAGE_ID}`")
        })?;
    let report_map_rows = report_map
        .into_iter()
        .filter(|row| row.stage_id == FASTQ_VALIDATE_READS_STAGE_ID)
        .collect::<Vec<_>>();
    ensure_required_tool_set(
        "fastq validate-reads report-map rows",
        report_map_rows.iter().map(|row| row.tool_id.as_str()),
    )?;

    let output_by_binding = output_rows
        .into_iter()
        .map(|row| (binding_key(&row.stage_id, &row.tool_id), row))
        .collect::<BTreeMap<_, _>>();
    let parser_by_binding = parser_rows
        .into_iter()
        .map(|row| (binding_key(&row.stage_id, &row.tool_id), row))
        .collect::<BTreeMap<_, _>>();
    let expected_by_binding = expected_rows
        .into_iter()
        .map(|row| (binding_key(&row.stage_id, &row.tool_id), row))
        .collect::<BTreeMap<_, _>>();
    let command_by_binding = command_rows
        .into_iter()
        .map(|row| (binding_key(&row.stage_id, &row.tool_id), row))
        .collect::<BTreeMap<_, _>>();
    let report_map_by_binding = report_map_rows
        .into_iter()
        .map(|row| (binding_key(&row.stage_id, &row.tool_id), row))
        .collect::<BTreeMap<_, _>>();

    Ok(AdmittedBindings {
        active_rows,
        output_by_binding,
        parser_by_binding,
        expected_by_binding,
        command_by_binding,
        report_map_by_binding,
        comparable_row,
        smoke,
    })
}
