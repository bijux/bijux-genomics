# bijux-dna-environment Architecture

`bijux-dna-environment` owns environment facts: tool-image metadata, platform resolution,
runtime compatibility, cache path derivation, and reference registration. It must stay below API,
CLI, runner, planner, stage, and domain crates.

## Source Layout

- `src/lib.rs`: crate root and stable module exports.
- `src/public_api/`: facade re-export surface for consumers that need one stable import path.
- `src/build/`: curated Docker tool defaults and Dockerfile version parsing.
- `src/resolve/types/`: serialized models, runtime enum, platform records, and environment errors.
- `src/resolve/catalog/`: TOML catalog loading, registry digest hydration, and image reference
  synthesis.
- `src/resolve/platform.rs`: platform-file loading and runner fallback selection.
- `src/resolve/cache/`: cache roots, Docker image inspection hooks, and Apptainer/Singularity SIF
  path derivation.
- `src/resolve/reference/`: reference-copy registration, digest records, and optional local index
  preparation.
- `src/resolve/commands.rs`, `shell.rs`, `smoke.rs`: bounded host-command helpers documented in
  `COMMANDS.md`.
- `src/runtime_spec/`: pure pairing of a selected runner with a resolved platform.

## Data Flow

1. Build metadata supplies known tool defaults or extracts versions from Dockerfiles.
2. Platform TOML selects runner, architecture, image prefix, and container directory rules.
3. Image catalog TOML provides tool versions and optional digests.
4. Registry pins can hydrate missing digests from the local registry file.
5. Resolver functions return `ResolvedImage`, `PlatformSpec`, `RuntimeSpec`, or reference records.

## Boundaries

- The crate may inspect local host capabilities and declared cache paths.
- The crate may prepare reference index files only when explicitly requested by
  `ReferenceBuildRequest`.
- The crate must not own biological stage execution, planner semantics, user CLI routing, report
  rendering, or runner backends.
- Runtime QA orchestration belongs in `bijux-dna-environment-qa`; this crate exposes compatibility
  helpers that the QA crate may call.

## Change Rules

- Add a source directory only when it has a stable ownership concern.
- Update `docs/PUBLIC_API.md` when `src/lib.rs` exports change.
- Update `docs/COMMANDS.md` before adding or removing any `std::process::Command` use.
- Update boundary tests when the tree changes intentionally.
