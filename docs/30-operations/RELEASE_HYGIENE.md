# Release Hygiene

## What
Rules for contract versioning and documentation updates.

## Why
Prevents breaking changes without explicit updates.

## Non-goals
- Release automation.

## Contracts
Breaking contract change requires:
- docs update
- snapshot update
- version bump

## Examples
If `RunManifest` changes, update schema snapshots + `CONTRACT_VERSIONING.md`.

## Failure modes
CI fails if snapshots or docs are missing.
