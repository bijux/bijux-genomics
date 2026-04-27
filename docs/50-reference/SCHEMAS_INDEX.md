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
See [crates/bijux-dna-api/tests/snapshots/](../../crates/bijux-dna-api/tests/snapshots).

## CLI help
See [crates/bijux-dna/tests/snapshots/](../../crates/bijux-dna/tests/snapshots).

## Contract schemas
See [crates/bijux-dna-core/tests/schemas/](../../crates/bijux-dna-core/tests/schemas) and
[crates/bijux-dna-stage-contract/tests/schemas/](../../crates/bijux-dna-stage-contract/tests/schemas).

## Failure modes
- Missing references lead to silent contract drift.
