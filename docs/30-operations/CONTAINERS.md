# Containers

## What
Defines container image naming and build rules.

## Why
Container digests make execution reproducible.

## Non-goals
- Managing registries for the user.

## Contracts
- images.toml must include digests for production runs.

## Examples
- `bijuxdna/fastp:0.23.4-arm64` with immutable digest.

## Failure modes
- Missing digest blocks promotion to production.
