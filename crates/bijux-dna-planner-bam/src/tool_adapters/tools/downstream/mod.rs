//! Downstream tool adapter implementations for BAM pipelines.

pub mod authenticity;
pub mod contammix;
pub mod gatk;
pub mod rxy;
pub mod schmutzi;
pub mod verifybamid2;

pub const DOWNSTREAM_TOOL_IDS: &[&str] = &[
    "authenticct",
    "contammix",
    "gatk",
    "rxy",
    "schmutzi",
    "verifybamid2",
];
