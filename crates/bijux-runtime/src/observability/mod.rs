//! Observability and reporting contracts.
//!
//! Boundaries:
//! - Schema definitions only (no IO, no network).
//! - No heavy dependencies; keep this module lightweight and stable.

mod contracts;
mod reports;
mod telemetry;

pub use contracts::*;
pub use reports::*;
pub use telemetry::*;
