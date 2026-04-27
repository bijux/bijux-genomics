# Version Authority

Purpose: define the source of truth for container versions and source pins.

[../README.md](../README.md) defines the broader container control surface, and
[../versions/index.md](../versions/index.md) is the versioning entrypoint this
authority governs.

## Authority Order
1. [containers/versions/versions.toml](../versions/versions.toml) is the only
   editable version authority.
2. [containers/versions/lock.json](../versions/lock.json) is generated from
   versions + build manifests under the rules in
   [containers/versions/LOCK.md](../versions/LOCK.md).
3. Container defs and Dockerfiles must reference the version authority contract.

## Required Fields
Each tool entry must include:
- `version`
- `source`
- one of `source_sha256` or `pinned_commit`

## Enforcement
- `cargo run -p bijux-dna-dev -- containers run check-version-hash-pin`
- `cargo run -p bijux-dna-dev -- containers run check-version-authority`
- `cargo run -p bijux-dna-dev -- containers run check-version-completeness`
