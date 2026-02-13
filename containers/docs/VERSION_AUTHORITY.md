# Version Authority

Purpose: define the source of truth for container versions and source pins.

## Authority Order
1. `containers/versions/versions.toml` is the only editable version authority.
2. `containers/versions/lock.json` is generated from versions + build manifests.
3. Container defs and Dockerfiles must reference the version authority contract.

## Required Fields
Each tool entry must include:
- `version`
- `source`
- one of `source_sha256` or `pinned_commit`

## Enforcement
- `scripts/containers/check-version-hash-pin.sh`
- `scripts/containers/check-version-authority.sh`
- `scripts/containers/check-version-completeness.sh`
