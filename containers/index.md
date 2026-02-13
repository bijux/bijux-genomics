# Containers Index

Purpose: Define container taxonomy, authority, and version ownership for reproducible execution.

## Apptainer vs Docker
- `containers/apptainer/`: Apptainer definitions for HPC and isolate-first runs.
- `containers/docker/`: Dockerfiles for OCI image build and smoke validation.
- Docker architecture contract: this repository currently ships `containers/docker/arm64/` definitions only.
- Multiarch contract details: `containers/docker/multiarch-policy.md`.

## Bijux vs Non-bijux
- `containers/apptainer/bijux/`: Bijux-maintained definitions with project policy headers.
- `containers/apptainer/non-bijux/`: Third-party sourced definitions tracked with explicit upstream provenance in `containers/apptainer/non-bijux/NON_BIJUX_SOURCES.md`.

## Versions And Authority
- Canonical version pins: `containers/versions/versions.toml`.
- Pin/lock policy: `containers/versions/LOCK.md`.
- Planned backlog and justification: `containers/PLANNED.md`.
- Operational guide: `docs/30-operations/CONTAINERS.md`.

## Tool Identity Contract
- Allowed tool IDs for container filenames are generated in `containers/TOOL_IDS.txt`.
- Regenerate with `scripts/containers/generate-tool-ids.sh`.

## Notes
- `seqkit_stats` is intentionally modeled as a distinct tool ID and stage binding (`fastq.stats_neutral`) in registry and container definitions.
