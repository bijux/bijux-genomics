# Container Promotion Policy

Purpose: Define required gates for status transitions (`planned` -> `experimental` -> `stable`) and demotion.

[../README.md](../README.md), [../index.md](../index.md), and
[VERSION_AUTHORITY.md](VERSION_AUTHORITY.md) define the adjacent container
surfaces this promotion policy depends on.

## Promotion Gates
0. Ownership/provenance classification:
   - `bijux` means Bijux owns and maintains the container recipe in this repo.
   - `non-bijux` means recipe remains upstream-derived with minimal packaging-only adaptation.
   - Classification is about recipe ownership/provenance, not license type.
1. License clarity:
   - Registry entry has a known license.
   - Non-bijux tools are present in
     [containers/apptainer/shared/NON_BIJUX_SOURCES.md](../apptainer/shared/NON_BIJUX_SOURCES.md).
2. Provenance:
   - Upstream source URL and checksum are recorded.
   - Version row exists in [containers/versions/versions.toml](../versions/versions.toml).
3. Reproducibility:
   - [containers/versions/LOCK.md](../versions/LOCK.md) is regenerated and its
     governed `lock.json` matches.
   - Container lint and lock checks pass.
4. Smoke quality:
   - `--help` and `--version` smoke checks pass.
   - Minimal invocation smoke is green where defined.

## Status Transitions
- `planned -> experimental`: use `cargo run -p bijux-dna-dev -- containers run tool-lifecycle -- --tool <id> --to experimental`.
- `experimental -> stable`: use `cargo run -p bijux-dna-dev -- containers run tool-lifecycle -- --tool <id> --to stable` only after all gates pass.
- `stable -> experimental`: use `cargo run -p bijux-dna-dev -- containers run demote -- --tool <id> --stage <domain.stage> --reason <text> --removal-after <YYYY-MM-DD>`.

Implementation note:
- Registry status value remains `production` for stable tools; `stable` is the lifecycle command alias.
- Manual edits to registry status fields are forbidden; use native lifecycle commands only.

## Ownership
- Tool ownership is tracked in [containers/OWNERS.toml](../OWNERS.toml).
- Owner must review promotions and demotions.
