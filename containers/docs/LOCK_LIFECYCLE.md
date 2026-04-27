# Lock Lifecycle

Purpose: define how [containers/versions/lock.json](../versions/lock.json) is
produced and validated.

[../README.md](../README.md), [VERSION_AUTHORITY.md](VERSION_AUTHORITY.md), and
[../versions/LOCK.md](../versions/LOCK.md) define the broader control surface
that this workflow enforces.

## Flow
1. Edit [containers/versions/versions.toml](../versions/versions.toml).
2. Generate lock: `cargo run -p bijux-dna-dev -- containers run generate-version-lock`.
3. Validate lock schema and drift checks.
4. Use lock for promotion and release gating.

## Generated-Only Rule
- [containers/versions/lock.json](../versions/lock.json) must be generated.
- `generator_script` and `generator_sha256` in lock must match generator script.

## Integrity Checks
- `cargo run -p bijux-dna-dev -- containers run check-lock-schema`
- `cargo run -p bijux-dna-dev -- containers run check-lock-drift`
- `cargo run -p bijux-dna-dev -- containers run check-lock-change-discipline`
- `cargo run -p bijux-dna-dev -- containers run check-version-lock`
