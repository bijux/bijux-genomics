//! Recording and runtime emit helpers.
//!
//! Boundaries:
//! - Only write under the run layout.
//! - No heavy dependencies; keep this module lightweight and stable.

mod io;
mod manifests;
mod metrics;
mod provenance;
mod telemetry;

pub use io::*;
pub use manifests::*;
pub use metrics::*;
pub use provenance::*;
pub use telemetry::*;
