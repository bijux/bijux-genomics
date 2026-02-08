# LEGACY (Debt Ledger)

## Why it exists
Legacy benchmarks support older datasets and tools.

Status: frozen. Legacy support is a deprecation path only; no new features or schema changes
should land in `legacy/`.

## What remains
- Frozen legacy fixtures required for historical comparisons.
- A small compatibility layer to load legacy artifacts.

## Why it remains
Some external consumers still depend on legacy artifacts; removing them would break
historical comparisons.

## Removal plan
- Migrate remaining consumers to the current benchmark suite.
- Remove legacy loaders once no consumers remain for two minor releases.
- Delete legacy fixtures after a final verification run.

## What it blocks
- Simplified schema unification.
- Removal of deprecated metrics.

## Replacement
Use current benchmark suite under `bench` module.
