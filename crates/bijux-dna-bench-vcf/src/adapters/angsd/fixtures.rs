//! Owner: bijux-dna-bench-vcf
//! angsd adapter fixtures contracts.
use super::{
    artifact, path_relative_to_repo, Context, Path, Result, VcfAngsdAdapterArtifact,
    VcfDomainStage, GOVERNED_REFERENCE_FASTA_PATH, GOVERNED_SITES_VCF_PATH,
};

pub(super) fn governed_inputs_for_stage(
    stage: VcfDomainStage,
    bam_list_path: Option<&str>,
) -> Result<Vec<VcfAngsdAdapterArtifact>> {
    Ok(match stage {
        VcfDomainStage::CallGl | VcfDomainStage::CallPseudohaploid => vec![
            artifact(
                "bam_list",
                "bam_list",
                bam_list_path.context("BAM-backed ANGSD stage requires a BAM list path")?,
            ),
            artifact("reference_fasta", "reference", GOVERNED_REFERENCE_FASTA_PATH),
            artifact("sites_vcf", "sites_vcf", GOVERNED_SITES_VCF_PATH),
        ],
        other => {
            return Err(super::anyhow!(
                "VCF angsd adapter does not govern stage `{}`",
                other.as_str()
            ));
        }
    })
}

pub(super) fn materialize_bam_list(
    repo_root: &Path,
    output_root: &str,
    bam_member_paths: &[String],
) -> Result<String> {
    let bam_list_path = repo_root.join(output_root).join("angsd-inputs.bam.list");
    if let Some(parent) = bam_list_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    let payload = format!("{}\n", bam_member_paths.join("\n"));
    bijux_dna_infra::atomic_write_bytes(&bam_list_path, payload.as_bytes())?;
    Ok(path_relative_to_repo(repo_root, &bam_list_path))
}
