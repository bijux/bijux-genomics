//! Core tool adapter implementations for BAM pipelines.

pub mod mapdamage2;
pub mod mosdepth;
pub mod preseq;
pub mod pydamage;

pub const CORE_TOOL_IDS: &[&str] = &["mapdamage2", "mosdepth", "preseq", "pydamage"];
