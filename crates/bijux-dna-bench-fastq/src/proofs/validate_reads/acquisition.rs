use super::smoke::load_fastq_validate_reads_smoke_evidence;
use super::source_inputs::SourceEvidence;
use crate::proofs::inputs::ValidationContext;
use anyhow::Result;
pub(super) fn acquire(context: &ValidationContext<'_>) -> Result<SourceEvidence> {
    let repo_root = context.repo_root;
    let active_matrix = context.declarations.active(repo_root)?;
    let output_contract = context.declarations.outputs(repo_root)?;
    let parser_coverage = context.declarations.parsers(repo_root)?;
    let comparable_metrics = context.validation.metrics(repo_root)?;
    let report_map = context.declarations.report_map(repo_root)?;
    let expected_results = context.declarations.expected(repo_root)?;
    let command_argv = context.validation.argv(repo_root)?;
    let smoke = load_fastq_validate_reads_smoke_evidence(context)?;

    Ok(SourceEvidence {
        active_matrix,
        output_contract,
        parser_coverage,
        comparable_metrics,
        report_map,
        expected_results,
        command_argv,
        smoke,
    })
}
