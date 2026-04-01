//! Deterministic environment resolution and digest pinning.
//!
//! Responsibilities: resolve platform + image catalog into pinned digests.
//! Invariants: same inputs produce identical resolved specs; no network pulls.

use std::path::PathBuf;

mod cache;
mod catalog;
mod commands;
mod facade;
mod platform;
mod reference;
mod shell;
mod smoke;
mod types;

pub use commands::{available_runners, docker_image_exists};
pub use facade::{
    apptainer_sif_path, cache_dir, load_image_catalog, load_platform, resolve_image,
    run_shell_capture, run_smoke_script, run_smoke_script_batch, select_best_runner,
    validate_images_for_stage, EnvironmentResolver,
};
pub use reference::{ReferenceBuildRequest, ReferenceRecord, ReferenceRegistry};
pub use types::{
    EnvError, ImageRef, PlatformSpec, ResolvedImage, RuntimeKind, ToolImageCatalog, ToolImageSpec,
};

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
