# Container Release Checklist

Purpose: define the mandatory gate before tagging a container-affecting release.

[../README.md](../README.md), [../index.md](../index.md),
[VERSION_AUTHORITY.md](VERSION_AUTHORITY.md), and
[GHCR_PUBLISH.md](GHCR_PUBLISH.md) define the adjacent surfaces this checklist
must close before a publish run.

## Preconditions
- Registry updates are merged from the governed
  [configs/ci/registry/](../../configs/ci/registry/).
- Version metadata and lock are updated
  ([containers/versions/versions.toml](../versions/versions.toml),
  [containers/versions/LOCK.md](../versions/LOCK.md)).
- Planned/production status transitions are tracked via promotion/demotion scripts.

## Required Commands
1. `cargo run -p bijux-dna-dev -- containers run ensure-images -- --plan`
2. `cargo run -q -p bijux-dna-dev -- containers run lint`
3. `cargo run -p bijux-dna-dev -- containers run container-doctor -- --strict`
4. `cargo run -p bijux-dna-dev -- containers run release-gate`
5. `cargo run -q -p bijux-dna-dev -- containers run generate-ghcr-publish-matrix -- artifacts/containers/ghcr/docker-arm64-publish-matrix.json --status production`
6. `cargo run -q -p bijux-dna-dev -- containers run generate-ghcr-apptainer-publish-matrix -- artifacts/containers/ghcr/apptainer-publish-matrix.json --status production`

## Required Artifacts
- `artifacts/containers/ensure-images/report.json`
- `artifacts/containers/summary.json` (or isolate-scoped equivalent)
- `artifacts/containers/doctor/report.json`
- `artifacts/containers/ghcr/docker-arm64-publish-matrix.json`
- `artifacts/containers/ghcr/apptainer-publish-matrix.json`
- smoke logs/manifests under `artifacts/containers/`

## Exit Criteria
- All release-gate checks pass with zero policy failures.
- Lock, smoke, provenance, and docs checks are green.
- GHCR runtime-family package scope is reviewed against
  [containers/docs/GHCR_PUBLISH.md](GHCR_PUBLISH.md) before a manual publish
  run.
- Container docs stay aligned with runtime contracts
  ([containers/README.md](../README.md), [containers/docs/index.md](index.md)).
