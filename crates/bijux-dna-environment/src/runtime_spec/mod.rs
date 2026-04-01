//! Runtime spec wiring for resolved environments.
//!
//! Responsibilities: pair resolved platform and runner choices.
//! Invariants: no side effects; pure configuration container.

mod compatibility;
mod model;

pub use compatibility::is_platform_runner_compatible;
pub use model::RuntimeSpec;
