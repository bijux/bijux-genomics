# Golden Assets

## What
Deterministic expected outputs used by contract and integration checks.

## Rules
- Organize by contract ID under `assets/golden/<contract-id>/...`.
- Each directory containing golden files must have `GENERATE.md` or `GENERATE.toml`.
- Golden outputs must be reproducible from committed inputs/scripts and not hand-edited.
- Toy-run golden bundles must include `artifact_checksums.json` for integrity checks.

## Retention Policy
- Keep only actively consumed contract/gate bundles.
- Remove superseded bundles after replacement is validated in CI.
- Historical bundles require explicit rationale in commit message and `GENERATE.md`.

## Update Workflow
1. Regenerate via `./scripts/run.sh assets refresh-golden`.
2. Verify integrity checks pass (`check-asset-checksums`, `check-assets-drift`).
3. Review `artifacts/assets-refresh/golden/report.json` for deterministic inputs/outputs.
4. Commit generated bundle plus report-driven changes together.
