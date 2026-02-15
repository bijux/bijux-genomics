//! Downstream tool adapter implementations for BAM pipelines.

pub mod authenticity;
pub mod contammix;
pub mod gatk;
pub mod genotyping;
pub mod kinship;
pub mod rxy;
pub mod schmutzi;
pub mod verifybamid2;

#[must_use]
pub const fn module_name() -> &'static str {
    "downstream"
}
