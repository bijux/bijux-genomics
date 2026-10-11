use anyhow::{bail, Context, Result};
use bijux_dna_bench_bam_proofs::ports::{NativeProofKind, NativeProofWriter};
use std::path::{Path, PathBuf};
pub(super) struct NativeAdapter;
impl NativeProofWriter for NativeAdapter {
    fn write(&self, root: &Path, kind: NativeProofKind) -> Result<PathBuf> {
        let active = crate::commands::support::workspace_root::resolve_repo_root()?;
        if active.canonicalize().context("resolve native proof repository")?
            != root.canonicalize().context("resolve requested proof repository")?
        {
            bail!("native BAM proof repository differs from the requested evidence root");
        }
        match kind {
            #[cfg(feature = "bam_downstream")]
            NativeProofKind::Genotyping => {
                bijux_dna_api::v1::api::bam::write_local_genotyping_plan()
            }
            #[cfg(feature = "bam_downstream")]
            NativeProofKind::Kinship => {
                bijux_dna_api::v1::api::bam::write_local_kinship_smoke_report()
            }
            #[cfg(not(feature = "bam_downstream"))]
            NativeProofKind::Genotyping | NativeProofKind::Kinship => {
                bail!("requested BAM native proof requires bam_downstream")
            }

            NativeProofKind::Authenticity => {
                bijux_dna_api::v1::api::bam::write_local_authenticity_smoke_report()
            }
            NativeProofKind::Contamination => {
                bijux_dna_api::v1::api::bam::write_local_contamination_smoke_report()
            }
            NativeProofKind::Damage => {
                bijux_dna_api::v1::api::bam::write_local_damage_smoke_report()
            }
            NativeProofKind::Haplogroups => {
                bijux_dna_api::v1::api::bam::write_local_haplogroups_smoke_report()
            }
            NativeProofKind::Recalibration => {
                bijux_dna_api::v1::api::bam::write_local_recalibration_smoke_report()
            }
            NativeProofKind::Sex => {
                bijux_dna_api::v1::api::bam::write_local_sex_tool_smoke_report()
            }
        }
    }
}

#[cfg(test)]
mod tests;
