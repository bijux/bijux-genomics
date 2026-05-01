use crate::fastq::{
    fastq_adna_profile, fastq_amplicon_standard_profile, fastq_amplicon_umi_profile,
    fastq_contaminant_depletion_profile, fastq_default_profile, fastq_edna_metabarcoding_profile,
    fastq_host_depletion_profile, fastq_minimal_profile, fastq_qc_only_profile,
    fastq_reference_adna_profile, fastq_rrna_depletion_profile, fastq_trim_qc_profile,
    fastq_umi_profile,
};
use crate::PipelineProfile;

#[must_use]
pub fn fastq_profiles() -> Vec<PipelineProfile> {
    vec![
        fastq_amplicon_standard_profile(),
        fastq_amplicon_umi_profile(),
        fastq_contaminant_depletion_profile(),
        fastq_default_profile(),
        fastq_edna_metabarcoding_profile(),
        fastq_host_depletion_profile(),
        fastq_minimal_profile(),
        fastq_qc_only_profile(),
        fastq_adna_profile(),
        fastq_reference_adna_profile(),
        fastq_rrna_depletion_profile(),
        fastq_trim_qc_profile(),
        fastq_umi_profile(),
    ]
}
