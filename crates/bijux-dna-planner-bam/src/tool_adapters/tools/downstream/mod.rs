//! Downstream tool adapter implementations for BAM pipelines.

pub mod angsd_sex;
pub mod authenticity;
pub mod authenticity_signal;
pub mod contamination;
pub mod gatk;
pub mod genotyping;
pub mod haplogroups;
pub mod kinship;
pub mod rxy;
pub use contamination::{contammix, schmutzi, verifybamid2};

#[must_use]
pub const fn module_name() -> &'static str {
    "downstream"
}
