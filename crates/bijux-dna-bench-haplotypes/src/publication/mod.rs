//! Retain immutable execution evidence and publish stable artifact addresses.
mod admission;
mod model;
mod paths;
mod prior;
mod write;
pub(crate) use admission::{execution_parent, PublicationPlan};
pub(crate) use model::{PublishedArtifactPaths, ReportOwner};
mod context;
pub(crate) use context::ReportContext;
mod stage_result;
