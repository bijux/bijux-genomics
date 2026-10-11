//! Owner: typed command artifact report admission and count reconciliation.
use crate::contract::validate_rendered_command_observations;
use crate::{
    AllDomainRenderedCommandRow, AllDomainRenderedCommandsReport, BenchError,
    ALL_DOMAIN_RENDERED_COMMANDS_SCHEMA_VERSION,
};
use std::collections::{BTreeMap, BTreeSet};

/// Build an admitted command publication report; does not attest governed slice completeness.
///
/// # Errors
/// Rejects malformed command rows before any publication effect.
pub fn build_rendered_command_artifact_report(
    rows: Vec<AllDomainRenderedCommandRow>,
    output_path: String,
    argv_output_path: String,
) -> Result<AllDomainRenderedCommandsReport, BenchError> {
    validate_rendered_command_observations(&rows)?;
    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut benchmark_status_counts = BTreeMap::<String, usize>::new();
    let mut command_source_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        *benchmark_status_counts.entry(row.benchmark_status.clone()).or_default() += 1;
        *command_source_counts.entry(row.command_source.clone()).or_default() += 1;
    }

    Ok(AllDomainRenderedCommandsReport {
        schema_version: ALL_DOMAIN_RENDERED_COMMANDS_SCHEMA_VERSION.to_string(),
        output_path,
        argv_output_path,
        row_count: rows.len(),
        result_id_count: rows
            .iter()
            .map(|row| row.result_id.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        domain_counts,
        benchmark_status_counts,
        command_source_counts,
        rows,
    })
}
