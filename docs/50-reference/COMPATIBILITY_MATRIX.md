# COMPATIBILITY_MATRIX

## What
Compatibility matrix for plan, manifest, and report versions.

## Why
Clarifies which artifacts can be safely combined.

## Non-goals
- Exhaustive version history.

## Contracts
- Breaking changes require major version increments.

## Examples
| Plan | Manifest | Report | Compatibility |
| --- | --- | --- | --- |
| v1 | v1 | v1 | compatible |

Version bumps: breaking changes require major increments.

## Failure modes
- Untracked version changes invalidate stored artifacts.
