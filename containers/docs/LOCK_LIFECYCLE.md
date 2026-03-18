# Lock Lifecycle

Purpose: define how `containers/versions/lock.json` is produced and validated.

## Flow
1. Edit `containers/versions/versions.toml`.
2. Generate lock: `cargo run -p bijux-dev-dna -- containers run generate-version-lock`.
3. Validate lock schema and drift checks.
4. Use lock for promotion and release gating.

## Generated-Only Rule
- `containers/versions/lock.json` must be generated.
- `generator_script` and `generator_sha256` in lock must match generator script.

## Integrity Checks
- `scripts/containers/check-lock-schema.sh`
- `scripts/containers/check-lock-drift.sh`
- `scripts/containers/check-lock-change-discipline.sh`
- `cargo run -p bijux-dev-dna -- containers run check-version-lock`
