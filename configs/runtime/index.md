# configs/runtime

## What
Configuration files for the runtime domain.

## Philosophy
Keep runtime configuration scoped to this directory so ownership is explicit and drift is easy to detect.

## platforms.toml Schema
- File: `configs/runtime/platforms.toml`
- Purpose: declare named execution platforms consumed by runtime/environment resolution.
- Invariant: every platform key is unique and stable over time.
- Invariant: each platform entry must define deterministic runtime selection (no floating aliases).
- Invariant: values are policy inputs only; they must not contain host-local absolute paths.

Fields:
- `platforms.<id>.runtime`: runtime selector (for example `docker-arm64`, `docker-amd64`, `apptainer`).
- `platforms.<id>.default`: optional boolean; at most one platform can be marked default.
- `platforms.<id>.notes`: optional human-readable rationale for platform selection constraints.

## Files
- `configs/runtime/platforms.toml`
- `configs/runtime/coverage_regimes.toml`
- `configs/runtime/species_aliases.toml`
- `configs/runtime/cargo_build.toml`
- `configs/runtime/profiles/index.md`

## profiles/ Layout
- Path: `configs/runtime/profiles/<profile>.toml`
- Purpose: composable runtime execution profiles selected by profile name (for example `local`, `hpc`).
- Invariant: profile file names are stable identifiers used by CLI `--profile`.
- Contract doc: `configs/runtime/profiles/README.md`.
