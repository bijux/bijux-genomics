//! BAM stage specs, metrics, and observers.

pub mod metrics;
pub mod observer;
mod plugin;
pub mod stage_specs;
mod surface;

pub use plugin::BamStagePlugin;
pub use surface::{implemented_stages, StagePlanJson};
