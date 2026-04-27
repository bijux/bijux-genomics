# GHCR Container Publication

Purpose: define how `bijux-genomics` publishes governed external-tool container packages to GitHub Container Registry.

[../README.md](../README.md), [../index.md](../index.md),
[VERSION_AUTHORITY.md](VERSION_AUTHORITY.md), and
[RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md) define the adjacent container
control surfaces this publish path must satisfy.

## Scope

- Repository packages view: `https://github.com/bijux?tab=packages&repo_name=bijux-genomics`
- Repository link target: `https://github.com/bijux/bijux-genomics`
- Runtime families:
  - Docker arm64 OCI images
  - Apptainer SIF artifacts stored in GHCR through the `oras://` protocol

The package namespace must follow the same repository-scoped principle used in sibling repositories such as `bijux-canon`, `bijux-pollenomics`, and `bijux-proteomics`: packages live under `ghcr.io/bijux/bijux-genomics/<package_slug>`, not under a flat organization namespace.

## Package Topology

- Docker arm64 package ref: `ghcr.io/bijux/bijux-genomics/docker-arm64-<tool_id>`
- Apptainer package ref: `ghcr.io/bijux/bijux-genomics/apptainer-<tool_id>`
- Docker package page:
  `https://github.com/bijux/bijux-genomics/pkgs/container/bijux-genomics%2Fdocker-arm64-<tool_id>`
- Apptainer package page:
  `https://github.com/bijux/bijux-genomics/pkgs/container/bijux-genomics%2Fapptainer-<tool_id>`

This split keeps runtime identity explicit:

- Docker arm64 packages are the OCI images consumed with `docker pull`.
- Apptainer packages are the SIF artifacts consumed with `apptainer pull oras://...`.

Do not publish both runtime families under the same package slug.

## Workflows

- Docker arm64 workflow:
  [publish-ghcr-container-images.yml](../../.github/workflows/publish-ghcr-container-images.yml)
- Apptainer workflow:
  [publish-ghcr-apptainer-images.yml](../../.github/workflows/publish-ghcr-apptainer-images.yml)

Both workflows are manual release surfaces guarded by `enabled=true`.

## Publication Inputs

Common manual inputs:

- `enabled`: required safety gate for actual publication
- `tool_ids`: optional comma or space separated subset
- `status_filter`: optional comma or space separated lifecycle filter
- `package_prefix`: defaults to `ghcr.io/bijux/bijux-genomics`
- `push_latest`: applies only to production tools

## Published Tags

Each package gets:

- `<tool_version>`
- `sha-<git-sha-prefix>`
- `latest` only when both of these are true:
  - the workflow input `push_latest` is enabled
  - the governed tool status is `production`

## Authority

- Docker arm64 matrix:
  `cargo run -q -p bijux-dna-dev -- containers run generate-ghcr-publish-matrix -- artifacts/containers/ghcr/docker-arm64-publish-matrix.json`
- Apptainer matrix:
  `cargo run -q -p bijux-dna-dev -- containers run generate-ghcr-apptainer-publish-matrix -- artifacts/containers/ghcr/apptainer-publish-matrix.json`
- Tool versions:
  [containers/versions/versions.toml](../versions/versions.toml)
- Tool runtime coverage:
  [configs/ci/registry/](../../configs/ci/registry/)
- Docker build surface:
  [containers/docker/arm64/](../docker/arm64/)
- Apptainer build surface:
  [containers/apptainer/shared/](../apptainer/shared/)
- Non-bijux provenance:
  [containers/apptainer/shared/NON_BIJUX_SOURCES.md](../apptainer/shared/NON_BIJUX_SOURCES.md)

## Pull Contract

- Docker arm64:
  `docker pull ghcr.io/bijux/bijux-genomics/docker-arm64-fastp:<tag>`
- Apptainer:
  `apptainer pull fastp.sif oras://ghcr.io/bijux/bijux-genomics/apptainer-fastp:<tag>`

Private packages require GHCR read access. Docker pulls use standard GHCR authentication. Apptainer pulls use `apptainer registry login --username <user> oras://ghcr.io` or equivalent token-backed credentials.

## Verification Surface

- Docker arm64 matrix artifact:
  `artifacts/containers/ghcr/docker-arm64-publish-matrix.json`
- Apptainer matrix artifact:
  `artifacts/containers/ghcr/apptainer-publish-matrix.json`
- Docker arm64 workflow results:
  `artifacts/containers/ghcr/workflow/docker-arm64/<tool_id>.json`
  `artifacts/containers/ghcr/workflow/docker-arm64/<tool_id>.metadata.json`
- Apptainer workflow results:
  `artifacts/containers/ghcr/workflow/apptainer/<tool_id>.json`

## Non-goals

- Publishing ungated tools outside registry authority
- Publishing Docker and Apptainer outputs under a shared ambiguous package name
- Treating repo-scoped release bundles from `release-ghcr.yml` as substitutes for runtime tool packages
- Hand-editing `container_ref` state without real package digests and published evidence
