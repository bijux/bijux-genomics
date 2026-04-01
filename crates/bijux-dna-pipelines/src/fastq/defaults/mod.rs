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
pub(crate) use defaults_assembly::fastq_defaults;
pub(crate) use reference_adna::reference_adna_fastq_defaults;

pub(crate) use stage_order::{append_stage_once, default_shotgun_required_stages};
