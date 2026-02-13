# Container Entrypoints

Purpose: Define the only supported user-facing container script entrypoints.

Supported entrypoints:
- `scripts/containers/ensure-images.sh`
- `scripts/containers/smoke-apptainer.sh`
- `scripts/containers/smoke-docker-arm64.sh`
- `scripts/containers/smoke-docker-amd64.sh`

All other scripts under `scripts/containers/` are internal helpers or policy checks.
