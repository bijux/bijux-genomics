# Container Entrypoints

Purpose: Define the only supported user-facing container script entrypoints.

Supported entrypoints:
- `scripts/containers/ensure-images.sh`
- `scripts/containers/smoke-apptainer.sh`
- `scripts/containers/smoke-docker-arm64.sh`
- `scripts/containers/smoke-docker-amd64.sh`

All other scripts under `scripts/containers/` are internal helpers or policy checks.

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
