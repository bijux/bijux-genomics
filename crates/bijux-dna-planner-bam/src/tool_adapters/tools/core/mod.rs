//! Core tool adapter implementations for BAM pipelines.

pub mod addeam;
pub mod damageprofiler;
pub mod mapdamage2;
pub mod mosdepth;
pub mod ngsbriggs;
pub mod pmdtools;
pub mod preseq;
pub mod pydamage;

pub const CORE_TOOL_IDS: &[&str] = &[
    "addeam",
    "damageprofiler",
    "mapdamage2",
    "mosdepth",
    "ngsbriggs",
    "preseq",
    "pmdtools",
    "pydamage",
];
