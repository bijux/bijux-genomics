use super::observations::ExpectedResultsObservation;
use super::observations::LocalSmokeObservation;
use super::observations::MicroSmokeObservation;
use super::observations::RenderedCommandsObservation;
use super::observations::ScienceThresholdsObservation;
use super::observations::ToolMapObservation;
use super::RetainedContext;
use anyhow::Result;
use bijux_dna_bench_bam_evidence::command_coverage::BamCommandAdapterCoverageReport;
use bijux_dna_bench_bam_evidence::consistency::BamActiveRowConsistencyReport;
use bijux_dna_bench_bam_evidence::output_contract::BamAdapterOutputContractReport;
use bijux_dna_bench_bam_evidence::parser_coverage::BamParserFixtureCoverageReport;
use bijux_dna_bench_bam_evidence::report_map::BamReportMapReport;
pub(super) struct EvidenceSet {
    pub tool_map: ToolMapObservation,
    pub command_coverage: BamCommandAdapterCoverageReport,
    pub parser_coverage: BamParserFixtureCoverageReport,
    pub local_smoke: LocalSmokeObservation,
    pub output_declarations: BamAdapterOutputContractReport,
    pub rendered_commands: RenderedCommandsObservation,
    pub expected_results: ExpectedResultsObservation,
    pub report_map: BamReportMapReport,
    pub active_row_consistency: BamActiveRowConsistencyReport,
    pub micro_smoke_summary: MicroSmokeObservation,
    pub science_thresholds: ScienceThresholdsObservation,
}
pub(super) fn acquire(context: &RetainedContext<'_>) -> Result<EvidenceSet> {
    let repo_root = context.repo_root;
    let tool_map = context.bindings.tool_map(repo_root)?;
    let command_coverage = context.bindings.command_coverage(repo_root)?;
    let parser_coverage = context.bindings.parser_coverage(repo_root)?;
    let local_smoke = context.bindings.local_smoke(repo_root)?;
    let output_declarations = context.bindings.output_declarations(repo_root)?;
    let rendered_commands = context.bindings.rendered_commands(repo_root)?;
    let expected_results = context.bindings.expected_results(repo_root)?;
    let report_map = context.bindings.report_map(repo_root)?;
    let active_row_consistency = context.qualification.active_row_consistency(repo_root)?;
    let micro_smoke_summary = context.qualification.micro_smoke_summary(repo_root)?;
    let science_thresholds = context.qualification.science_thresholds(repo_root)?;
    Ok(EvidenceSet {
        tool_map,
        command_coverage,
        parser_coverage,
        local_smoke,
        output_declarations,
        rendered_commands,
        expected_results,
        report_map,
        active_row_consistency,
        micro_smoke_summary,
        science_thresholds,
    })
}
