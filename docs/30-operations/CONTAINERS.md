# Containers

## What
Defines container image naming and build rules.

## Why
Container digests make execution reproducible.

## Non-goals
- Managing registries for the user.

## Contracts
- images.toml must include digests for production runs.
- Docker definitions are `arm64`-only unless policy and checks are updated.
- Container filenames must match generated `containers/TOOL_IDS.txt`.

## Examples
- `bijuxdna/fastp:0.23.4-arm64` with immutable digest.

## Failure modes
- Missing digest blocks promotion to production.

## References
- `containers/index.md`
- `containers/README.md`
