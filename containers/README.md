# Container Entrypoints

Purpose: Define the only supported user-facing container script entrypoints.

Supported entrypoints:
- `scripts/containers/ensure-images.sh`
- `scripts/containers/smoke-apptainer.sh`
- `scripts/containers/smoke-docker-arm64.sh`
- `scripts/containers/smoke-docker-amd64.sh`

All other scripts under `scripts/containers/` are internal helpers or policy checks.

## Runtime Expectations
- CPU: ARM64 is the default supported Docker architecture; Apptainer definitions are architecture-aware and validated via smoke contracts.
- Java tools: Java-based tools (for example Beagle/EIGENSOFT family) require a working JVM inside the container image; smoke checks validate version/help contracts.
- Memory profile: downstream population-genetics tools may require multi-GB memory; treat smoke as contract validation, not full workload sizing.
- HPC pull naming: image references for HPC pulls are generated from `configs/ci/tools/hpc_image_naming.toml` and validated by `scripts/containers/check-hpc-image-naming.sh`.
- Cache policy: Apptainer cache/temp roots are controlled by `configs/ci/tools/apptainer_cache_policy.toml`; non-isolated runs must not write cache under the repo tree.
- Bundle completeness: toolkit bundle definitions in `configs/ci/tools/toolkit_bundles.toml` are enforced by `scripts/containers/check-toolkit-bundles.sh`.

## How To Add A Tool
- Admission policy: `docs/50-reference/TOOL_ADMISSION.md`
- Promotion/demotion gates: `containers/PROMOTION_POLICY.md`
- Add/update registry rows for the tool in `configs/ci/registry/*.toml`.
- Add container defs (`containers/apptainer/...` and optionally `containers/docker/arm64/...`).
- Build targeted image smoke plan: `./scripts/containers/ensure-images.sh --plan --only <tool-id>`
- Run smoke: `./scripts/run.sh containers smoke-apptainer` and/or `./scripts/run.sh containers smoke-docker-arm64`
- Promote lifecycle:
  - `./scripts/containers/promote.sh --tool <tool-id> --to experimental`
  - `./scripts/containers/promote.sh --tool <tool-id> --to production`
- Demote with rationale:
  - `./scripts/containers/demote.sh --tool <tool-id> --stage <domain.stage> --reason \"...\" --removal-after YYYY-MM-DD`

## Release Readiness
- Checklist: `containers/RELEASE_CHECKLIST.md`
- Gate script: `./scripts/containers/release-gate.sh`
