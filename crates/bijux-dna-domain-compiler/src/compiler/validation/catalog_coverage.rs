use super::{bail, BTreeMap, BTreeSet, Result};

pub(super) fn validate_canonical_stage_coverage(
    stage_ids: &BTreeMap<String, String>,
) -> Result<()> {
    let fastq_canonical = bijux_dna_domain_fastq::stages::ids::STAGES
        .iter()
        .map(|id| id.as_str().to_string())
        .collect::<BTreeSet<_>>();
    let bam_canonical = bijux_dna_domain_bam::stage_specs::BamStage::all()
        .iter()
        .map(|stage| stage.as_str().to_string())
        .collect::<BTreeSet<_>>();
    let vcf_canonical = bijux_dna_domain_vcf::VCF_STAGE_ID_CATALOG
        .iter()
        .map(|stage_id| stage_id.to_string())
        .collect::<BTreeSet<_>>();

    for stage_id in &fastq_canonical {
        if !stage_ids.contains_key(stage_id) {
            bail!("fastq stage catalog contains {stage_id} but domain yaml is missing it");
        }
    }
    for stage_id in &bam_canonical {
        if !stage_ids.contains_key(stage_id) {
            bail!("bam stage catalog contains {stage_id} but domain yaml is missing it");
        }
    }
    for stage_id in &vcf_canonical {
        if !stage_ids.contains_key(stage_id) {
            bail!("vcf stage catalog contains {stage_id} but domain yaml is missing it");
        }
    }
    Ok(())
}
