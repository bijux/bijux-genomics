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
