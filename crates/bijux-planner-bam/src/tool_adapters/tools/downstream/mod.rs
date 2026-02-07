//! Downstream tool adapter implementations for BAM pipelines.

pub mod authenticity;
pub mod gatk;
pub mod rxy;

pub const DOWNSTREAM_TOOL_IDS: &[&str] = &["authenticct", "gatk", "rxy"];
