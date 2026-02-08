//! Owner: bijux-analyze
//! Report model builders and renderers.
//! Owns report model construction and rendering from typed inputs.
//! Must not depend on load; renderers must not query facts directly.
//! Invariants: renderers accept `ReportModel` only.

pub mod build;
pub(crate) mod model;
pub(crate) mod render;
pub(crate) mod sections;

mod bench;

pub use build::*;
