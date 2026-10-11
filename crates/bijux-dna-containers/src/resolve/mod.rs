//! Deterministic environment resolution and digest pinning.
//!
//! Responsibilities: resolve platform + image catalog into pinned digests.
//! Invariants: same inputs produce identical resolved specs; no network pulls.

use std::path::PathBuf;

mod cache;
mod catalog;
mod commands;
mod entrypoints;
mod facade;
mod platform;
mod reference;
mod shell;
mod smoke;
mod stable_surface;
mod types;

pub use stable_surface::*;

pub(crate) fn available_runners_with<F>(probe: F) -> Vec<RuntimeKind>
where
    F: Fn(&str) -> bool,
{
    platform::available_runners_with(probe)
}

pub(crate) fn docker_image_exists_with<F>(image: &ResolvedImage, runner: F) -> bool
where
    F: Fn(&[&str]) -> bool,
{
    cache::docker_image_exists_with(image, runner)
}

#[must_use]
pub fn reference_cache_dir() -> PathBuf {
    cache::reference_cache_dir()
}
