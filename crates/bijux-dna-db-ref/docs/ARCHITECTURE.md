# Architecture

The crate provides deterministic lookups for species context and reference bundles, and exposes lock-aware metadata for planner/runtime checks.

Source layout:
- `models.rs` defines reference and species domain models.
- `catalog.rs` defines panel and map catalog plus lock metadata.
- `config.rs` owns runtime TOML loading and internal config DTOs.
- `service.rs` exposes runtime resolver traits and the default service implementation.
- `resolution/` splits species resolution, reference assets, panel lookup, map lookup, and imputation compatibility policy into focused modules.
