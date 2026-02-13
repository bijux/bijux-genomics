# Container Promotion Policy

Purpose: Define required gates for status transitions (`planned` -> `experimental` -> `production`) and demotion.

## Promotion Gates
0. Ownership/provenance classification:
   - `bijux` means Bijux owns and maintains the container recipe in this repo.
   - `non-bijux` means recipe remains upstream-derived with minimal packaging-only adaptation.
   - Classification is about recipe ownership/provenance, not license type.
1. License clarity:
   - Registry entry has a known license.
   - Non-bijux tools are present in `containers/apptainer/non-bijux/NON_BIJUX_SOURCES.md`.
2. Provenance:
   - Upstream source URL and checksum are recorded.
   - Version row exists in `containers/versions/versions.toml`.
3. Reproducibility:
   - `containers/versions/lock.json` is regenerated and matches.
   - Container lint and lock checks pass.
4. Smoke quality:
   - `--help` and `--version` smoke checks pass.
   - Minimal invocation smoke is green where defined.

## Status Transitions
- `planned -> experimental`: use `scripts/containers/promote.sh --tool <id> --to experimental`.
- `experimental -> production`: use `scripts/containers/promote.sh --tool <id> --to production` only after all gates pass.
- `production -> experimental`: use `scripts/containers/demote.sh --tool <id> --stage <domain.stage> --reason <text> --removal-after <YYYY-MM-DD>`.

## Ownership
- Tool ownership is tracked in `containers/OWNERS.toml`.
- Owner must review promotions and demotions.
