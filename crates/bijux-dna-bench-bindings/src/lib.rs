//! Canonical benchmark binding evidence with explicit domain and planner adapters.
mod context;
pub mod expected;
pub mod observations;
pub mod ports;
mod publication;
pub mod serving;
pub use context::{ExpectedContext, ServingContext};
