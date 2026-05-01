use anyhow::anyhow;
use bijux_dna_core::prelude::id_catalog;

use super::{
    fastq_adna_profile, fastq_amplicon_standard_profile, fastq_amplicon_umi_profile,
    fastq_contaminant_depletion_profile, fastq_default_profile, fastq_edna_metabarcoding_profile,
    fastq_host_depletion_profile, fastq_minimal_profile, fastq_qc_only_profile,
    fastq_reference_adna_profile, fastq_rrna_depletion_profile, fastq_trim_qc_profile,
    fastq_umi_profile,
};
use crate::PipelineProfile;

/// # Errors
/// Returns an error if the requested profile id is unknown.
pub fn fastq_profiles_by_id(id: &str) -> anyhow::Result<PipelineProfile> {
    match id {
        id_catalog::PIPELINE_FASTQ_AMPLICON_STANDARD => Ok(fastq_amplicon_standard_profile()),
        id_catalog::PIPELINE_FASTQ_AMPLICON_UMI => Ok(fastq_amplicon_umi_profile()),
        id_catalog::PIPELINE_FASTQ_DEFAULT => Ok(fastq_default_profile()),
        id_catalog::PIPELINE_FASTQ_CONTAMINANT_DEPLETION => {
            Ok(fastq_contaminant_depletion_profile())
        }
        id_catalog::PIPELINE_FASTQ_EDNA_METABARCODING => Ok(fastq_edna_metabarcoding_profile()),
        id_catalog::PIPELINE_FASTQ_HOST_DEPLETION => Ok(fastq_host_depletion_profile()),
        id_catalog::PIPELINE_FASTQ_MINIMAL => Ok(fastq_minimal_profile()),
        id_catalog::PIPELINE_FASTQ_QC_ONLY => Ok(fastq_qc_only_profile()),
        id_catalog::PIPELINE_FASTQ_ADNA => Ok(fastq_adna_profile()),
        id_catalog::PIPELINE_FASTQ_REFERENCE_ADNA => Ok(fastq_reference_adna_profile()),
        id_catalog::PIPELINE_FASTQ_RRNA_DEPLETION => Ok(fastq_rrna_depletion_profile()),
        id_catalog::PIPELINE_FASTQ_TRIM_QC => Ok(fastq_trim_qc_profile()),
        id_catalog::PIPELINE_FASTQ_UMI => Ok(fastq_umi_profile()),
        _ => Err(anyhow!("unknown FASTQ profile: {id}")),
    }
}
