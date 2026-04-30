//! FASTQ pipeline profiles and defaults.

mod defaults;

pub mod invariants;
pub mod profiles;
pub mod workflow_registry;

pub use invariants::{
    validate_fastq_profile, FastqProfileValidationReport, FastqProfileViolation, FASTQ_INVARIANTS,
};
pub use profiles::{
    fastq_adna_profile, fastq_amplicon_standard_profile, fastq_amplicon_umi_profile,
    fastq_contaminant_depletion_profile, fastq_default_profile, fastq_edna_metabarcoding_profile,
    fastq_host_depletion_profile, fastq_minimal_profile, fastq_profiles_by_id,
    fastq_qc_only_profile, fastq_reference_adna_profile, fastq_rrna_depletion_profile,
    fastq_trim_qc_profile, fastq_umi_profile, FASTQ_PROFILE_IDS,
};
pub use workflow_registry::{
    fastq_workflow_template_by_id, fastq_workflow_templates, fastq_workflow_templates_for_pipeline,
};
