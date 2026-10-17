//! Owner: bijux-dna-bench-artifacts
//! Retain immutable execution evidence and publish stable artifact addresses.
mod admission;
mod model;
mod paths;
mod prior;
mod write;
pub use admission::{execution_parent, PublicationPlan};
pub use model::{PublicationSchema, PublishedArtifactPaths, ReportOwner};
mod contract;
mod stage_result;

pub mod schemas;

pub mod results;

mod staged;
pub use staged::StagedPublicationPlan;
