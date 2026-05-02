#![allow(
    clippy::case_sensitive_file_extension_comparisons,
    clippy::collapsible_if,
    clippy::let_and_return,
    clippy::map_unwrap_or,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::needless_return,
    clippy::needless_pass_by_value,
    clippy::too_many_lines,
    clippy::redundant_closure_for_method_calls,
    clippy::unnecessary_lazy_evaluations,
    clippy::verbose_bit_mask
)]

mod appraisers;
mod bam_campaign;
mod benchmark_matrix;
mod bundle;
mod campaign;
mod cross_campaign;
mod fastq_campaign;
mod hardening_campaign;
mod layout;
mod preparation;
mod slurm;
mod vcf_campaign;

pub use appraisers::*;
pub use bam_campaign::*;
pub use benchmark_matrix::*;
pub use bundle::*;
pub use campaign::*;
pub use cross_campaign::*;
pub use fastq_campaign::*;
pub use hardening_campaign::*;
pub use layout::*;
pub use preparation::*;
pub use slurm::*;
pub use vcf_campaign::*;
