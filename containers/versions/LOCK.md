# Container Version Lock Rules

Purpose: Define lock semantics for `containers/versions/versions.toml` and `containers/versions/lock.json`.

## Authority
- Canonical version source: `containers/versions/versions.toml`.
- Canonical lock artifact: `containers/versions/lock.json`.
- Lock generator: `scripts/containers/generate-version-lock.sh`.

## What `lock.json` Pins
- `schema_version`: lock schema contract version (`bijux.container.version_lock.v2`).
- `source`: canonical source path (`containers/versions/versions.toml`).
- `source_sha256`: full-file hash of `versions.toml`.
- `items[].tool`: tool ID.
- `items[].version`: canonical declared version extracted from `versions.toml`.
- `items[].status`: lifecycle status.
- `items[].entry_sha256`: hash of canonicalized per-tool version entry.

## Pin Meaning
- A pin is a reviewed version/provenance entry in `versions.toml`.
- Floating references are forbidden (`latest`, branch names, empty version fields).
- Any edit to `versions.toml` must regenerate and commit `lock.json`.

## Update Workflow
1. Update container definition(s) and registry records.
2. Update `containers/versions/versions.toml`.
3. Regenerate lock: `./scripts/containers/generate-version-lock.sh`.
4. Validate authority and drift:
   - `./scripts/containers/check-version-authority.sh`
   - `./scripts/containers/check-version-lock.sh`
5. Commit changes together with rationale.

## Deprecation Workflow
- Version deprecations are tracked in `containers/versions/deprecations.toml`.
- Add a deprecation entry via:
  - `./scripts/containers/deprecate-version.sh --tool <id> --version <v> --rationale <text> --removal-after YYYY-MM-DD`
- Validation gate:
  - `./scripts/containers/check-version-deprecations.sh`
- Reproducibility rule: deprecated versions must remain represented in lock metadata until compatibility window closes.
