//! Core tool adapter implementations for BAM pipelines.
//! This module exists as the owned aggregation root for core BAM tool adapters.

pub mod addeam;
pub mod damageprofiler;
pub mod mapdamage2;
pub mod mosdepth;
pub mod ngsbriggs;
pub mod picard;
pub mod pmdtools;
pub mod preseq;
pub mod pydamage;

#[must_use]
pub const fn module_name() -> &'static str {
    "core"
}
