use anyhow::{bail, Context, Result};
use bijux_dna_bench_bam_evidence::readiness::ports::{LocalProofKind, LocalProofWriter};
use std::path::{Path, PathBuf};
pub(super) struct NativeAdapter;
impl LocalProofWriter for NativeAdapter {
    fn write(&self, root: &Path, kind: LocalProofKind) -> Result<PathBuf> {
        // Existing native writers discover their active root. Refuse a different request
        // before their effects; do not mutate process-global root variables to redirect them.
        let active = crate::commands::support::workspace_root::resolve_repo_root()?;
        if active.canonicalize().context("resolve native proof repository")?
            != root.canonicalize().context("resolve requested proof repository")?
        {
            bail!("native BAM proof repository differs from the requested evidence root");
        }
        match kind {
            LocalProofKind::ContaminationPlan => {
                bijux_dna_api::v1::api::bam::write_local_contamination_plan()
            }
            LocalProofKind::SexSmoke => bijux_dna_api::v1::api::bam::write_local_sex_smoke_report(),
            #[cfg(feature = "bam_downstream")]
            LocalProofKind::HaplogroupsPlan => {
                bijux_dna_api::v1::api::bam::write_local_haplogroups_plan()
            }
            LocalProofKind::RecalibrationSmoke => {
                bijux_dna_api::v1::api::bam::write_local_recalibration_smoke_report()
            }
            #[cfg(feature = "bam_downstream")]
            LocalProofKind::GenotypingPlan => {
                bijux_dna_api::v1::api::bam::write_local_genotyping_plan()
            }
            LocalProofKind::DamageSmoke => {
                bijux_dna_api::v1::api::bam::write_local_damage_smoke_report()
            }
            LocalProofKind::AuthenticitySmoke => {
                bijux_dna_api::v1::api::bam::write_local_authenticity_smoke_report()
            }
            LocalProofKind::InsertSizeSmoke => {
                bijux_dna_api::v1::api::bam::write_local_insert_size_smoke_report()
            }
            LocalProofKind::GcBiasSummary => {
                bijux_dna_api::v1::api::bam::write_local_gc_bias_smoke_summary()
            }
            LocalProofKind::EndogenousContentSmoke => {
                bijux_dna_api::v1::api::bam::write_local_endogenous_content_smoke_report()
            }
            LocalProofKind::OverlapCorrectionSmoke => {
                bijux_dna_api::v1::api::bam::write_local_overlap_correction_smoke_report()
            }
            #[cfg(feature = "bam_downstream")]
            LocalProofKind::KinshipSmoke => {
                bijux_dna_api::v1::api::bam::write_local_kinship_smoke_report()
            }
            #[cfg(not(feature = "bam_downstream"))]
            LocalProofKind::HaplogroupsPlan
            | LocalProofKind::GenotypingPlan
            | LocalProofKind::KinshipSmoke => {
                bail!("requested BAM native proof requires bam_downstream")
            }
        }
    }
}

#[cfg(test)]
mod tests;
