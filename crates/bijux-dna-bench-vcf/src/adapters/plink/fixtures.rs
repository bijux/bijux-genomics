//! Owner: bijux-dna-bench-vcf
//! plink adapter fixtures contracts.
use super::{artifact, VcfDomainStage, VcfPlinkFamilyAdapterArtifact, GOVERNED_COHORT_VCF_PATH};

pub(super) fn governed_inputs_for_stage(
    stage: VcfDomainStage,
    sample_metadata_path: &str,
) -> Vec<VcfPlinkFamilyAdapterArtifact> {
    let mut inputs = vec![artifact("vcf", "variant", GOVERNED_COHORT_VCF_PATH)];
    if matches!(
        stage,
        VcfDomainStage::Pca | VcfDomainStage::Admixture | VcfDomainStage::PopulationStructure
    ) {
        inputs.push(artifact("sample_metadata_manifest", "sample_metadata", sample_metadata_path));
    }
    inputs
}
