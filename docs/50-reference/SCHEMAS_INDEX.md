# SCHEMAS_INDEX

## What
Index of schema snapshots that define external contracts.

## Why
Provides a single place to locate enforced schema references.

## Non-goals
- Describing schema contents in detail.

## Contracts
- Schema snapshots listed here must remain stable unless explicitly versioned.

## Examples
## API schemas
See `crates/bijux-api/tests/snapshots/*`.

## CLI help
See `crates/bijux-cli/tests/snapshots/*`.

## Contract schemas
See `crates/bijux-core/tests/*` and `crates/bijux-stage-contract/tests/*`.

## Failure modes
- Missing references lead to silent contract drift.
