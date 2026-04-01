//! Recording and runtime emit helpers.
//!
//! Boundaries:
//! - Only write under the run layout.
//! - No heavy dependencies; keep this module lightweight and stable.

mod envelope;
mod io;
mod manifests;
mod metrics;
mod provenance;
mod stable_surface;
mod telemetry;

pub use stable_surface::*;
