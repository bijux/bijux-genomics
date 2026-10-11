use anyhow::Result;
use bijux_dna_bench_bam_evidence::readiness::contamination_sex_haplogroups::BamContaminationSexHaplogroupsReadyReport;
use bijux_dna_bench_bam_evidence::readiness::damage_authenticity::BamDamageAuthenticityReadyReport;
use bijux_dna_bench_bam_evidence::readiness::overlap_endogenous::BamOverlapEndogenousReadyReport;
use bijux_dna_bench_bam_evidence::readiness::recalibration_genotyping::BamRecalibrationGenotypingReadyReport;
use bijux_dna_bench_bam_proofs::ports::ReadinessEvidence;
use std::path::Path;
pub(super) struct ReadinessAdapter;
impl ReadinessEvidence for ReadinessAdapter {
    fn damage_authenticity(&self, root: &Path) -> Result<BamDamageAuthenticityReadyReport> {
        use super::super::bam_damage_authenticity_ready::{
            render_bam_damage_authenticity_ready, DEFAULT_BAM_DAMAGE_AUTHENTICITY_READY_PATH,
        };
        render_bam_damage_authenticity_ready(
            root,
            DEFAULT_BAM_DAMAGE_AUTHENTICITY_READY_PATH.into(),
        )
    }
    fn contamination_sex_haplogroups(
        &self,
        root: &Path,
    ) -> Result<BamContaminationSexHaplogroupsReadyReport> {
        use super::super::bam_contamination_sex_haplogroups_ready::{
            render_bam_contamination_sex_haplogroups_ready,
            DEFAULT_BAM_CONTAMINATION_SEX_HAPLOGROUPS_READY_PATH,
        };
        render_bam_contamination_sex_haplogroups_ready(
            root,
            DEFAULT_BAM_CONTAMINATION_SEX_HAPLOGROUPS_READY_PATH.into(),
        )
    }
    fn overlap_endogenous(&self, root: &Path) -> Result<BamOverlapEndogenousReadyReport> {
        use super::super::bam_overlap_endogenous_ready::{
            render_bam_overlap_endogenous_ready, DEFAULT_BAM_OVERLAP_ENDOGENOUS_READY_PATH,
        };
        render_bam_overlap_endogenous_ready(root, DEFAULT_BAM_OVERLAP_ENDOGENOUS_READY_PATH.into())
    }
    fn recalibration_genotyping(
        &self,
        root: &Path,
    ) -> Result<BamRecalibrationGenotypingReadyReport> {
        use super::super::bam_recalibration_genotyping_ready::{
            render_bam_recalibration_genotyping_ready,
            DEFAULT_BAM_RECALIBRATION_GENOTYPING_READY_PATH,
        };
        render_bam_recalibration_genotyping_ready(
            root,
            DEFAULT_BAM_RECALIBRATION_GENOTYPING_READY_PATH.into(),
        )
    }
}
