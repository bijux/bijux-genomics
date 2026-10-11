#![cfg(test)]
use anyhow::{anyhow, bail, Result};
use bijux_dna_bench_bam_evidence::command_coverage::BamCommandAdapterCoverageReport;
use bijux_dna_bench_bam_evidence::consistency::BamActiveRowConsistencyReport;
use bijux_dna_bench_bam_evidence::output_contract::BamAdapterOutputContractReport;
use bijux_dna_bench_bam_evidence::parser_coverage::BamParserFixtureCoverageReport;
use bijux_dna_bench_bam_evidence::report_map::BamReportMapReport;
use bijux_dna_bench_bam_proofs::retained_tools::observations::ExpectedResultsObservation;
use bijux_dna_bench_bam_proofs::retained_tools::observations::LocalSmokeObservation;
use bijux_dna_bench_bam_proofs::retained_tools::observations::MicroSmokeObservation;
use bijux_dna_bench_bam_proofs::retained_tools::observations::RenderedCommandsObservation;
use bijux_dna_bench_bam_proofs::retained_tools::observations::ScienceThresholdsObservation;
use bijux_dna_bench_bam_proofs::retained_tools::observations::ToolMapObservation;
use bijux_dna_bench_bam_proofs::retained_tools::{
    ports::{BindingEvidence, QualificationEvidence, StageCompletion, StageProofKind, StageProofs},
    render_bam_all_retained_tools_complete, EvidencePaths, RetainedContext,
};
use std::{cell::Cell, path::Path};
struct Refused;
impl BindingEvidence for Refused {
    fn tool_map(&self, _root: &Path) -> Result<ToolMapObservation> {
        bail!("canonical tool-map unavailable")
    }
    fn command_coverage(&self, _root: &Path) -> Result<BamCommandAdapterCoverageReport> {
        bail!("unexpected command_coverage acquisition")
    }
    fn parser_coverage(&self, _root: &Path) -> Result<BamParserFixtureCoverageReport> {
        bail!("unexpected parser_coverage acquisition")
    }
    fn local_smoke(&self, _root: &Path) -> Result<LocalSmokeObservation> {
        bail!("unexpected local_smoke acquisition")
    }
    fn output_declarations(&self, _root: &Path) -> Result<BamAdapterOutputContractReport> {
        bail!("unexpected output_declarations acquisition")
    }
    fn rendered_commands(&self, _root: &Path) -> Result<RenderedCommandsObservation> {
        bail!("unexpected rendered_commands acquisition")
    }
    fn expected_results(&self, _root: &Path) -> Result<ExpectedResultsObservation> {
        bail!("unexpected expected_results acquisition")
    }
    fn report_map(&self, _root: &Path) -> Result<BamReportMapReport> {
        bail!("unexpected report_map acquisition")
    }
}
impl QualificationEvidence for Refused {
    fn active_row_consistency(&self, _root: &Path) -> Result<BamActiveRowConsistencyReport> {
        bail!("unexpected active_row_consistency acquisition")
    }
    fn micro_smoke_summary(&self, _root: &Path) -> Result<MicroSmokeObservation> {
        bail!("unexpected micro_smoke_summary acquisition")
    }
    fn science_thresholds(&self, _root: &Path) -> Result<ScienceThresholdsObservation> {
        bail!("unexpected science_thresholds acquisition")
    }
}
struct StageCalls(Cell<usize>);
impl StageProofs for StageCalls {
    fn assess(
        &self,
        _root: &Path,
        _kind: StageProofKind,
        _output: &Path,
    ) -> Result<StageCompletion> {
        self.0.set(self.0.get() + 1);
        bail!("unexpected stage proof")
    }
}
#[test]
fn source_failure_preserves_existing_aggregate_before_stage_proofs() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let output = dir.path().join("aggregate.json");
    std::fs::write(&output, b"prior aggregate")?;
    let stages = StageCalls(Cell::new(0));
    let context = RetainedContext {
        repo_root: dir.path(),
        bindings: &Refused,
        qualification: &Refused,
        stages: &stages,
        paths: EvidencePaths {
            local_smoke: "canonical-source",
            parser_coverage: "canonical-source",
            rendered_commands: "canonical-source",
            output_declarations: "canonical-source",
            expected_results: "canonical-source",
            report_map: "canonical-source",
            active_row_consistency: "canonical-source",
            micro_smoke_summary: "canonical-source",
            science_thresholds: "canonical-source",
        },
    };
    let error = render_bam_all_retained_tools_complete(&context, &output)
        .err()
        .ok_or_else(|| anyhow!("unavailable evidence accepted"))?;
    assert_eq!(error.to_string(), "canonical tool-map unavailable");
    assert_eq!(stages.0.get(), 0);
    assert_eq!(std::fs::read(&output)?, b"prior aggregate");
    Ok(())
}
