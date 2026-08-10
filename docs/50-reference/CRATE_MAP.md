# Crate Map

## What
Navigation to the governed crate ownership and boundary maps.

## Why
Provides quick architectural orientation without duplicating the normative responsibility table.

## Non-goals
- Deep API docs.

## Contracts
- Crate ownership and responsibilities are cataloged in
  [TARGET_ARCHITECTURE.md](../../TARGET_ARCHITECTURE.md).
- Allowed boundary edges are defined in
  [BOUNDARY_MAP.md](../10-architecture/BOUNDARY_MAP.md).
- Required enforcement level by crate maturity is defined in
  [MATURITY_LADDER.md](../40-policies/MATURITY_LADDER.md).

## Examples

- Use the target architecture to decide which crate owns a new product behavior.
- Use the boundary map to decide whether the required dependency edge is permitted.

## Failure modes

- Adding a second crate-role table here lets ownership drift from the target architecture.
- Treating role prose as a dependency allowlist bypasses the executable boundary map.
