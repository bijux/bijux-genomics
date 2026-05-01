//! FASTQ pipeline default construction.

mod adna;
mod analysis_params;
mod analysis_tools;
mod defaults_assembly;
mod parameter_defaults;
mod preprocess_params;
mod preprocess_tools;
mod rationales;
mod reference_adna;
mod stage_order;
mod tool_defaults;

pub(crate) use adna::adna_fastq_defaults;
pub(crate) use defaults_assembly::{fastq_defaults, generic_fastq_defaults};
pub(crate) use reference_adna::reference_adna_fastq_defaults;

pub(crate) use stage_order::{
    amplicon_standard_required_stages, amplicon_umi_required_stages, append_stage_once,
    contaminant_depletion_required_stages, default_shotgun_required_stages,
    edna_metabarcoding_required_stages, host_depletion_required_stages,
    minimal_shotgun_required_stages, qc_only_required_stages, rrna_depletion_required_stages,
    umi_required_stages,
};
