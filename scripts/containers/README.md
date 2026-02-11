## Container Helper Scripts

This directory is the single entrypoint for container helper scripts.

- `smoke-docker-arm64.sh`: build/smoke Docker arm64 images and write manifests.
- `smoke-docker-amd64.sh`: amd64 wrapper around the arm64 smoke logic.
- `smoke-apptainer.sh`: build/smoke Apptainer images and write manifests.
- `apptainer_build_all.sh`: batch Apptainer build helper for HPC/VM workflows.
- `lint.sh`: static checks for container definitions.
- `summary.sh`: summarize smoke manifest results.

Callers should reference scripts from this directory rather than `scripts/` root.
