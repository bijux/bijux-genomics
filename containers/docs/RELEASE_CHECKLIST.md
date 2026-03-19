# Container Release Checklist

Purpose: define the mandatory gate before tagging a container-affecting release.

## Preconditions
- Registry updates are merged (`configs/ci/registry/*.toml`).
- Version metadata and lock are updated (`containers/versions/versions.toml`, `containers/versions/lock.json`).
- Planned/production status transitions are tracked via promotion/demotion scripts.

## Required Commands
1. `cargo run -p bijux-dev-dna -- containers run ensure-images -- --plan`
2. `./scripts/run.sh containers lint`
3. `cargo run -p bijux-dev-dna -- containers run container-doctor -- --strict`
4. `cargo run -p bijux-dev-dna -- containers run release-gate`

## Required Artifacts
- `artifacts/containers/ensure-images/report.json`
- `artifacts/containers/summary.json` (or isolate-scoped equivalent)
- `artifacts/containers/doctor/report.json`
- smoke logs/manifests under `artifacts/containers/`

## Exit Criteria
- All release-gate checks pass with zero policy failures.
- Lock, smoke, provenance, and docs checks are green.
- Container docs stay aligned with runtime contracts (`containers/README.md`, `containers/docs/index.md`).
