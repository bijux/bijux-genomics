//! BAM stage specs, metrics, and observers.

pub mod metrics;
pub mod observer;
mod plugin;
mod surface;
pub mod stage_specs;

pub use surface::{implemented_stages, StagePlanJson};
