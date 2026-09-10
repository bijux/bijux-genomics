//! Owner: bijux-dna-bench-vcf
//! bcftools adapter fixtures contracts.
use super::{
    artifact, materialize_reference_fasta_with_index, Path, PathBuf, Result, VcfAdapterInputs,
    VcfBcftoolsAdapterArtifact, VcfDomainStage, GOVERNED_BAM_INDEX_PATH, GOVERNED_BAM_PATH,
    GOVERNED_FILTERED_SINGLE_SAMPLE_VCF_PATH, GOVERNED_MULTISAMPLE_VCF_PATH,
    GOVERNED_RAW_SINGLE_SAMPLE_VCF_PATH, GOVERNED_REFERENCE_FASTA_PATH,
    GOVERNED_REFERENCE_PANEL_VCF_PATH,
};

pub(super) fn governed_inputs_for_stage(
    inputs: &VcfAdapterInputs,
    repo_root: &Path,
    stage: VcfDomainStage,
    output_root: &str,
) -> Result<Vec<VcfBcftoolsAdapterArtifact>> {
    Ok(match stage {
        VcfDomainStage::Qc | VcfDomainStage::Stats => {
            vec![artifact("vcf", "variant", GOVERNED_MULTISAMPLE_VCF_PATH)]
        }
        VcfDomainStage::PrepareReferencePanel => {
            vec![artifact("reference_panel_vcf", "variant", GOVERNED_REFERENCE_PANEL_VCF_PATH)]
        }
        VcfDomainStage::Call
        | VcfDomainStage::CallDiploid
        | VcfDomainStage::CallGl
        | VcfDomainStage::CallPseudohaploid => {
            let (reference_fasta, reference_fai) = materialize_reference_fasta_with_index(
                inputs,
                repo_root,
                GOVERNED_REFERENCE_FASTA_PATH,
                &PathBuf::from(output_root).join("artifacts/reference"),
            )?;
            vec![
                artifact("input_bam", "bam", GOVERNED_BAM_PATH),
                artifact("input_bam_index", "index", GOVERNED_BAM_INDEX_PATH),
                artifact("reference_fasta", "reference", &reference_fasta),
                artifact("reference_fai", "index", &reference_fai),
            ]
        }
        VcfDomainStage::Postprocess => {
            vec![artifact("vcf", "variant", GOVERNED_FILTERED_SINGLE_SAMPLE_VCF_PATH)]
        }
        _ => vec![artifact("vcf", "variant", GOVERNED_RAW_SINGLE_SAMPLE_VCF_PATH)],
    })
}
