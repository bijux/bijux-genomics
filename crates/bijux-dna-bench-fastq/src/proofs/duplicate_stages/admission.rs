use super::binding::binding_key;
use super::binding::collect_rows_by_binding;
use super::contracts::duplicate_stage_binding_admitted;
use super::contracts::duplicate_stage_spec;
use super::contracts::ensure_duplicate_stage_tool_sets;
use super::contracts::ensure_required_stage_rows;
use super::source_inputs::AdmittedBindings;
use super::source_inputs::SourceEvidence;
use anyhow::Result;
use std::collections::BTreeMap;
pub(super) fn admit(source: SourceEvidence) -> Result<AdmittedBindings> {
    let SourceEvidence {
        active_matrix,
        output_contract,
        parser_coverage,
        expected_results,
        report_map,
        command_coverage,
        schema_report,
        schema_contracts,
    } = source;
    let active_rows = active_matrix
        .into_iter()
        .filter(|row| duplicate_stage_spec(&row.stage_id).is_some())
        .collect::<Vec<_>>();
    ensure_duplicate_stage_tool_sets(
        "FASTQ duplicate-stage active rows",
        active_rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    )?;

    let output_rows = output_contract
        .into_iter()
        .filter(|row| duplicate_stage_binding_admitted(&row.stage_id, &row.tool_id))
        .collect::<Vec<_>>();
    ensure_duplicate_stage_tool_sets(
        "FASTQ duplicate-stage output-contract rows",
        output_rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    )?;

    let parser_rows = parser_coverage
        .into_iter()
        .filter(|row| duplicate_stage_binding_admitted(&row.stage_id, &row.tool_id))
        .collect::<Vec<_>>();
    ensure_duplicate_stage_tool_sets(
        "FASTQ duplicate-stage parser rows",
        parser_rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    )?;

    let expected_rows = expected_results
        .into_iter()
        .filter(|row| {
            row.domain == "fastq" && duplicate_stage_binding_admitted(&row.stage_id, &row.tool_id)
        })
        .collect::<Vec<_>>();
    ensure_duplicate_stage_tool_sets(
        "FASTQ duplicate-stage expected-result rows",
        expected_rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    )?;

    let command_rows = command_coverage
        .into_iter()
        .filter(|row| duplicate_stage_binding_admitted(&row.stage_id, &row.tool_id))
        .collect::<Vec<_>>();
    ensure_duplicate_stage_tool_sets(
        "FASTQ duplicate-stage rendered-command rows",
        command_rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    )?;

    let report_map_rows = report_map
        .into_iter()
        .filter(|row| duplicate_stage_spec(&row.stage_id).is_some())
        .collect::<Vec<_>>();
    ensure_duplicate_stage_tool_sets(
        "FASTQ duplicate-stage report-map rows",
        report_map_rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    )?;
    ensure_required_stage_rows(
        "FASTQ duplicate-stage schema rows",
        schema_contracts.keys().map(String::as_str),
    )?;

    let output_by_binding = collect_rows_by_binding(output_rows)?;
    let parser_by_binding = collect_rows_by_binding(parser_rows)?;
    let expected_by_binding = collect_rows_by_binding(expected_rows)?;
    let command_by_binding = collect_rows_by_binding(command_rows)?;
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
        schema_report,
        schema_contracts,
    })
}
