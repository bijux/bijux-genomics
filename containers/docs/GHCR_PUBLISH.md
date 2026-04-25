# GHCR Container Publication

Purpose: define how `bijux-genomics` publishes governed Docker-backed tool images to GitHub Container Registry.

## Scope

- Publication target: `ghcr.io/bijux/<tool_id>`
- Repository link target: `https://github.com/bijux/bijux-genomics`
- Runtime scope: Docker-backed tool images only

Apptainer definitions are not OCI images. They are governed in this repository, but they are not published by this workflow unless they also have a Docker image surface.

## Package Topology

- Packages are organization-scoped under `bijux`.
- Packages must appear on the organization packages view filtered to `repo_name=bijux-genomics`.
- The publish workflow stamps `org.opencontainers.image.source=https://github.com/bijux/bijux-genomics` at build time so GitHub can link the package back to this repository.

The repository keeps upstream tool provenance in `containers/versions/versions.toml`, `configs/ci/registry/tool_registry*.toml`, and the governed container docs. GHCR publication metadata uses the repository link as the registry-facing source of truth.

## Publication Inputs

Workflow: `.github/workflows/publish-ghcr-container-images.yml`

Manual inputs:

- `enabled`: required safety gate for actual publication
- `tool_ids`: optional comma or space separated subset
- `status_filter`: optional comma or space separated lifecycle filter
- `package_prefix`: defaults to `ghcr.io/bijux`
- `push_latest`: applies only to production tools

## Published Tags

Each published image gets:

- `<tool_version>`
- `sha-<git-sha-prefix>`
- `latest` only when both of these are true:
  - the workflow input `push_latest` is enabled
  - the governed tool status is `production`

## Authority

- Publication matrix generator:
  `cargo run -q -p bijux-dna-dev -- containers run generate-ghcr-publish-matrix -- artifacts/containers/ghcr/publish-matrix.json`
- Tool versions:
  `containers/versions/versions.toml`
- Tool runtime coverage:
  `configs/ci/registry/tool_registry*.toml`
- Docker build surface:
  `containers/docker/arm64/Dockerfile.<tool_id>`

## Non-goals

- Publishing Apptainer `.def` files as OCI images
- Publishing SIF artifacts to GHCR
- Rewriting governed runtime `container_ref` entries before real digests exist

## Verification Surface

- Matrix artifact:
  `artifacts/containers/ghcr/publish-matrix.json`
- Workflow result artifacts:
  `artifacts/containers/ghcr/workflow/<tool_id>.json`
  `artifacts/containers/ghcr/workflow/<tool_id>.metadata.json`
