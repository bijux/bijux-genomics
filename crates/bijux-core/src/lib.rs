//! Bijux core contracts and foundation types.

#![deny(missing_docs)]

/// Contract bible (stable, serialized interfaces).
pub mod contract;
/// Foundation primitives for core (hashing, errors, IO-agnostic helpers).
pub(crate) mod foundation;
/// Canonical identifier catalog.
pub mod id_catalog;
/// Strongly typed identifiers.
pub mod ids;
/// Metrics types and registries.
pub mod metrics;
/// Public prelude for stable imports.
pub mod prelude;
