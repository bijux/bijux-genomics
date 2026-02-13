# Containers Index

Purpose: Define container taxonomy, authority, and version ownership for reproducible execution.

## Apptainer vs Docker
- `containers/apptainer/`: Apptainer definitions for HPC and isolate-first runs.
- `containers/docker/`: Dockerfiles for OCI image build and smoke validation.

## Bijux vs Non-bijux
- `containers/apptainer/bijux/`: Bijux-maintained definitions with project policy headers.
- `containers/apptainer/non-bijux/`: Third-party sourced definitions tracked with explicit upstream provenance in `containers/apptainer/non-bijux/NON_BIJUX_SOURCES.md`.

## Versions And Authority
- Canonical version pins: `containers/versions/versions.toml`.
- Pin/lock policy: `containers/versions/LOCK.md`.
- Operational guide: `docs/30-operations/CONTAINERS.md`.
