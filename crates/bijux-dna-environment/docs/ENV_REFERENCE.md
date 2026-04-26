# bijux-dna-environment Reference

This is the executable behavior map for environment resolution.

## Platform Matrix

Supported platform records are defined by `configs/runtime/platforms.toml`. The crate accepts any
platform entry that provides:

- `runner`: `docker`, `apptainer`, or `singularity`
- `container_dir`: runtime-specific local container directory
- `image_prefix`: image namespace or registry prefix
- `arch`: architecture suffix used in tag construction

Docker platforms keep their configured `container_dir`. Apptainer and Singularity platforms prefer:

1. `BIJUX_APPTAINER_CONTAINER_DIR`
2. `BIJUX_CACHE_ROOT/bijux-dna-container/apptainer/sif`
3. `BIJUX_HPC_ROOT/.cache/bijux-dna-container/apptainer/sif`
4. the configured `container_dir`

## Image Resolution

Input spec to resolved image:

1. Load `ToolImageSpec` from the local image catalog.
2. Hydrate missing digests from the local registry pin file when available.
3. Reject empty tool names, versions, digests, platform prefixes, and architectures.
4. Prefer digest form: `<image_prefix>/<tool>@<digest>`.
5. Otherwise emit tag form: `<image_prefix>/<tool>:<version>-<arch>`.

## Cache Semantics

Cache keys are derived from runner, tool, digest or version, and architecture. Digest changes and
platform changes produce different paths. Cache helpers compute or inspect local state; they do not
pull images.

## Fixtures

These examples correspond to `tests/contracts/matrix/reference_matrix.rs` fixtures:

- `tests/fixtures/env_schema/default/tool_image_spec.json`
- `tests/fixtures/env_schema/default/platform_spec.json`

## Threat Model

Stable resolution assumes local input files are trusted. Stability can be broken by changing catalog
contents, registry pin mappings, platform defaults, architecture suffixes, or precedence rules.
Network registry answers, mutable tags, and undeclared host state are not trusted resolution inputs.

## Not Supported

- Remote registry resolution during normal image resolution.
- HPC scheduler integration.
- Automatic schema migration for changed serialized fields.
