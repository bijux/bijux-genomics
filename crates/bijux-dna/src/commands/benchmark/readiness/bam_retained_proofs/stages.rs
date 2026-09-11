use super::super::bam_authenticity_complete;
use super::super::bam_contamination_complete;
use super::super::bam_damage_complete;
use super::super::bam_endogenous_content_complete;
use super::super::bam_genotyping_complete;
use super::super::bam_haplogroups_complete;
use super::super::bam_kinship_complete;
use super::super::bam_overlap_correction_complete;
use super::super::bam_recalibration_complete;
use super::super::bam_sex_complete;
use anyhow::Result;
use bijux_dna_bench_bam_proofs::retained_tools::ports::{
    StageCompletion, StageProofKind, StageProofs,
};
use std::path::Path;
pub(super) struct StageAdapter;
impl StageProofs for StageAdapter {
    fn assess(&self, root: &Path, kind: StageProofKind, output: &Path) -> Result<StageCompletion> {
        match kind {
            StageProofKind::OverlapCorrection => {
                let report =
                    bam_overlap_correction_complete::render_bam_overlap_correction_complete(
                        root,
                        output.to_path_buf(),
                    )?;
                Ok(StageCompletion {
                    complete_row_count: report.complete_row_count,
                    active_row_count: report.active_row_count,
                    checked_surface_count: report.checked_surface_count,
                })
            }
            StageProofKind::EndogenousContent => {
                let report =
                    bam_endogenous_content_complete::render_bam_endogenous_content_complete(
                        root,
                        output.to_path_buf(),
                    )?;
                Ok(StageCompletion {
                    complete_row_count: report.complete_row_count,
                    active_row_count: report.active_row_count,
                    checked_surface_count: report.checked_surface_count,
                })
            }
            StageProofKind::Damage => {
                let report =
                    bam_damage_complete::render_bam_damage_complete(root, output.to_path_buf())?;
                Ok(StageCompletion {
                    complete_row_count: report.complete_row_count,
                    active_row_count: report.active_row_count,
                    checked_surface_count: report.checked_surface_count,
                })
            }
            StageProofKind::Authenticity => {
                let report = bam_authenticity_complete::render_bam_authenticity_complete(
                    root,
                    output.to_path_buf(),
                )?;
                Ok(StageCompletion {
                    complete_row_count: report.complete_row_count,
                    active_row_count: report.active_row_count,
                    checked_surface_count: report.checked_surface_count,
                })
            }
            StageProofKind::Contamination => {
                let report = bam_contamination_complete::render_bam_contamination_complete(
                    root,
                    output.to_path_buf(),
                )?;
                Ok(StageCompletion {
                    complete_row_count: report.complete_row_count,
                    active_row_count: report.active_row_count,
                    checked_surface_count: report.checked_surface_count,
                })
            }
            StageProofKind::Sex => {
                let report = bam_sex_complete::render_bam_sex_complete(root, output.to_path_buf())?;
                Ok(StageCompletion {
                    complete_row_count: report.complete_row_count,
                    active_row_count: report.active_row_count,
                    checked_surface_count: report.checked_surface_count,
                })
            }
            StageProofKind::Haplogroups => {
                let report = bam_haplogroups_complete::render_bam_haplogroups_complete(
                    root,
                    output.to_path_buf(),
                )?;
                Ok(StageCompletion {
                    complete_row_count: report.complete_row_count,
                    active_row_count: report.active_row_count,
                    checked_surface_count: report.checked_surface_count,
                })
            }
            StageProofKind::Recalibration => {
                let report = bam_recalibration_complete::render_bam_recalibration_complete(
                    root,
                    output.to_path_buf(),
                )?;
                Ok(StageCompletion {
                    complete_row_count: report.complete_row_count,
                    active_row_count: report.active_row_count,
                    checked_surface_count: report.checked_surface_count,
                })
            }
            StageProofKind::Genotyping => {
                let report = bam_genotyping_complete::render_bam_genotyping_complete(
                    root,
                    output.to_path_buf(),
                )?;
                Ok(StageCompletion {
                    complete_row_count: report.complete_row_count,
                    active_row_count: report.active_row_count,
                    checked_surface_count: report.checked_surface_count,
                })
            }
            StageProofKind::Kinship => {
                let report =
                    bam_kinship_complete::render_bam_kinship_complete(root, output.to_path_buf())?;
                Ok(StageCompletion {
                    complete_row_count: report.complete_row_count,
                    active_row_count: report.active_row_count,
                    checked_surface_count: report.checked_surface_count,
                })
            }
        }
    }
}
