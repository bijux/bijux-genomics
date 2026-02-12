# Containers

## Scope
Defines the shared Docker and Apptainer label contract, naming policy, and build-script layout.

## Unified Label Schema
Every container definition must declare:
- `org.opencontainers.image.title`
- `org.opencontainers.image.version`
- `org.opencontainers.image.source`
- `org.opencontainers.image.license` (or `org.opencontainers.image.licenses`)

## Naming Policy
Use one container per tool identity. Mode differences must be expressed via CLI arguments, not separate container names.

## Build Script Layout
All container build/smoke/lint scripts must live under `scripts/containers/`.

## Contract
- Container metadata must satisfy the policy tests.
- Duplicate strict/diagnose style container variants are not allowed.

## Governance Rules
- Registry is the authority for container scope: smoke/build targets are resolved from registry runtime mappings, not static file lists.
- Promotion is performed via `bijux dna registry promote --tool <id>` and must update:
  - registry status for the tool
  - `containers/versions/versions.toml`
  - `artifacts/container_manifest.json`
- Production tools must have:
  - a pinned entry in `containers/versions/versions.toml`
  - successful smoke contract definitions (`smoke_version_cmd` and required help probe)
- Container/runtime parity is enforced both ways:
  - registry runtime tools must have matching container definitions
  - container definitions must not be orphaned outside registry
- Explicit version variables are mandatory:
  - Docker: `ARG TOOL_VERSION`
  - Apptainer: `%labels` entry `VERSION ...`
- Parallel container builds/smoke use `BIJUX_WORKERS`.
