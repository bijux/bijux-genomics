use super::observations::LocalKinshipSmokeCaseReport;
use super::observations::LocalKinshipSmokeReport;
use crate::downstream::observations::ParserObservation;
use crate::downstream::observations::PipelineNode;
use crate::downstream::observations::PipelineObservation;
use bijux_dna_bench_bam_evidence::readiness::kinship::BamKinshipReadyRow;
use std::path::Path;
pub(super) struct RowInputs<'a> {
    pub repo_root: &'a Path,
    pub smoke_path: &'a Path,
    pub smoke_report: &'a LocalKinshipSmokeReport,
    pub ready_case: &'a LocalKinshipSmokeCaseReport,
    pub insufficient_case: &'a LocalKinshipSmokeCaseReport,
    pub readiness_row: &'a BamKinshipReadyRow,
    pub parser_row: &'a ParserObservation,
    pub bam_pipeline: &'a PipelineObservation,
    pub bam_pipeline_node: &'a PipelineNode,
    pub vcf_pipeline: &'a PipelineObservation,
}
