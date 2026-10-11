//! Retry policy and backoff math for external attempt planning.

mod backoff;
mod policy;
mod stable_surface;

pub use stable_surface::*;
