use super::super::bam_kinship_ready::{render_bam_kinship_ready, DEFAULT_BAM_KINSHIP_READY_PATH};
use anyhow::Result;
use bijux_dna_bench_bam_evidence::readiness::kinship::BamKinshipReadyReport;
use bijux_dna_bench_bam_proofs::downstream::ports::KinshipReadiness;
use std::path::Path;
pub(super) struct KinshipAdapter;
impl KinshipReadiness for KinshipAdapter {
    fn kinship(&self, root: &Path) -> Result<BamKinshipReadyReport> {
        render_bam_kinship_ready(root, DEFAULT_BAM_KINSHIP_READY_PATH.into())
    }
}
