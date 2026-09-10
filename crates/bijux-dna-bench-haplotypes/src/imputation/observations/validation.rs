//! Validate imputed genotype, sample and indexed VCF semantics.
use crate::imputation::model::ImputeOutputSummary;
use crate::imputation::model::EXPECTED_DONOR_SAMPLE_GT;
use crate::imputation::model::EXPECTED_MASKED_SAMPLE_GT;
use crate::imputation::model::EXPECTED_SAMPLE_IDS;
use crate::imputation::model::EXPECTED_VARIANT_COUNT;
use crate::imputation::observations::variants::summarize_imputed_output;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_domain_vcf::artifacts::genotypes_equivalent_unphased;
use bijux_dna_stages_vcf::pipeline::assert_bgzip_tabix_artifacts;
use bijux_dna_stages_vcf::vcf_io::vcf_validate_input;
use bijux_dna_stages_vcf::vcf_io::VcfFieldRequirement;
use bijux_dna_stages_vcf::vcf_io::VcfValidationSummary;
use std::path::Path;

pub(crate) fn observe_variants(
    output_vcf_path: &Path,
    output_tbi_path: &Path,
) -> Result<(VcfValidationSummary, ImputeOutputSummary)> {
    assert_bgzip_tabix_artifacts(output_vcf_path, output_tbi_path)?;
    let validation = vcf_validate_input(
        output_vcf_path,
        VcfFieldRequirement { require_gt: true, require_gl: true },
    )
    .with_context(|| format!("validate {}", output_vcf_path.display()))?;
    let output_summary = summarize_imputed_output(output_vcf_path)?;
    if output_summary.variant_count != EXPECTED_VARIANT_COUNT {
        bail!(
            "governed VCF impute smoke expected {} variants, found {}",
            EXPECTED_VARIANT_COUNT,
            output_summary.variant_count
        );
    }
    if output_summary.sample_ids
        != EXPECTED_SAMPLE_IDS.iter().map(|value| (*value).to_string()).collect::<Vec<_>>()
    {
        bail!(
            "governed VCF impute smoke expected sample ids {:?}, found {:?}",
            EXPECTED_SAMPLE_IDS,
            output_summary.sample_ids
        );
    }
    if !genotypes_equivalent_unphased(EXPECTED_MASKED_SAMPLE_GT, &output_summary.masked_sample_gt) {
        bail!(
            "governed VCF impute smoke expected masked sample GT `{EXPECTED_MASKED_SAMPLE_GT}`, found `{}`",
            output_summary.masked_sample_gt
        );
    }
    if !genotypes_equivalent_unphased(EXPECTED_DONOR_SAMPLE_GT, &output_summary.donor_sample_gt) {
        bail!(
            "governed VCF impute smoke expected donor sample GT `{EXPECTED_DONOR_SAMPLE_GT}`, found `{}`",
            output_summary.donor_sample_gt
        );
    }

    Ok((validation, output_summary))
}
