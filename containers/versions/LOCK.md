# Container Version Lock Rules

Purpose: Define lock semantics for
[containers/versions/versions.toml](versions.toml) and
[containers/versions/lock.json](lock.json).

[../README.md](../README.md), [index.md](index.md),
[../docs/VERSION_AUTHORITY.md](../docs/VERSION_AUTHORITY.md), and
[../docs/FRONTEND_BUILD_AUTHORITY.md](../docs/FRONTEND_BUILD_AUTHORITY.md)
define the broader control surfaces this lock artifact must satisfy.

## Authority
- Canonical version source: [containers/versions/versions.toml](versions.toml).
- Canonical lock artifact: [containers/versions/lock.json](lock.json).
- Lock generator: `cargo run -p bijux-dna-dev -- containers run generate-version-lock`.

## What `lock.json` Pins
- `schema_version`: lock schema contract version (`bijux.container.version_lock.v3`).
- `source`: canonical source path (`containers/versions/versions.toml`).
- `source_sha256`: full-file hash of `versions.toml`.
- `items[].tool`: tool ID.
- `items[].version`: canonical declared version extracted from `versions.toml`.
- `items[].status`: lifecycle status.
- `items[].entry_sha256`: hash of canonicalized per-tool version entry.
- `items[].frontend_resolved_sif_sha256`: authoritative SIF digest built on HPC frontend (`artifacts/containers/hpc/frontend-sif-digests.json`).
- `items[].frontend_smoke_version_output_sha256`: hash of normalized frontend smoke `--version` output (`artifacts/containers/hpc/frontend-smoke/summary.json`).

## Pin Meaning
- A pin is a reviewed version/provenance entry in `versions.toml`.
- Floating references are forbidden (`latest`, branch names, empty version fields).
- Any edit to `versions.toml` must regenerate and commit `lock.json`.

## Update Workflow
1. Update container definition(s) and registry records.
2. Update `containers/versions/versions.toml`.
3. Regenerate lock: `cargo run -p bijux-dna-dev -- containers run generate-version-lock`.
4. Validate authority and drift:
   - `cargo run -p bijux-dna-dev -- containers run check-version-authority`
   - `cargo run -p bijux-dna-dev -- containers run check-version-lock`
5. Commit changes together with rationale.

## Deprecation Workflow
- Version deprecations are tracked in
  [containers/versions/deprecations.toml](deprecations.toml).
- Add a deprecation entry via:
  - `cargo run -p bijux-dna-dev -- containers run deprecate-version -- --tool <id> --version <v> --rationale <text> --sunset-date YYYY-MM-DD --replacement-tool <id> --replacement-version <v>`
- Validation gate:
  - `cargo run -p bijux-dna-dev -- containers run check-version-deprecations`
- Reproducibility rule: deprecated versions must remain represented in lock metadata until compatibility window closes.
