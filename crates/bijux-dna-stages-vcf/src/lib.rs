//! VCF stage specs and metrics parser bindings.
#![allow(
    clippy::assigning_clones,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::format_push_string,
    clippy::incompatible_msrv,
    clippy::if_not_else,
    clippy::inefficient_to_string,
    clippy::map_identity,
    clippy::map_unwrap_or,
    clippy::match_same_arms,
    clippy::missing_errors_doc,
    clippy::redundant_closure_for_method_calls,
    clippy::similar_names,
    clippy::too_many_lines,
    clippy::uninlined_format_args
)]

pub mod engine;
pub mod invariants;
pub mod metrics;
pub mod pipeline;
pub mod stage_specs;
pub mod vcf_io;
pub mod wrappers;

#[must_use]
pub fn implemented_stages() -> Vec<bijux_dna_domain_vcf::VcfStage> {
    bijux_dna_domain_vcf::VcfStage::all().to_vec()
}
