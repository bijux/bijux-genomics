use super::schema::collect_trim_stage_schema_contracts;
use super::source_inputs::SourceEvidence;
use crate::proofs::inputs::SchemaContext;
use anyhow::Result;
pub(super) fn acquire(context: &SchemaContext<'_>) -> Result<SourceEvidence> {
    let repo_root = context.repo_root;
    let active_matrix = context.declarations.active(repo_root)?;
    let output_contract = context.declarations.outputs(repo_root)?;
    let parser_coverage = context.declarations.parsers(repo_root)?;
    let expected_results = context.declarations.expected(repo_root)?;
    let report_map = context.declarations.report_map(repo_root)?;
    let command_coverage = context.schema.commands(repo_root)?;
    let schema_report = context.schema.publish(repo_root)?;
    let schema_contracts = collect_trim_stage_schema_contracts(context)?;

    Ok(SourceEvidence {
        active_matrix,
        output_contract,
        parser_coverage,
        expected_results,
        report_map,
        command_coverage,
        schema_report,
        schema_contracts,
    })
}
