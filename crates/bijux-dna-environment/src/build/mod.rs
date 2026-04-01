//! Environment catalog build helpers.
//!
//! Responsibilities: derive tool metadata from dockerfiles and curated defaults.
//! Invariants: no resolution side effects; outputs must be deterministic for the same inputs.

mod builder;
mod defaults;
mod models;
mod version_parser;

pub use builder::{default_docker_tools, extract_version_from_dockerfile, EnvironmentBuilder};
pub use models::DockerToolSpec;
