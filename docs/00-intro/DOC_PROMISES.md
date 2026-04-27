# DOC_PROMISES

## What
Each section declares what is guaranteed to be accurate and how it is enforced.

## Why
Docs must be contract-backed, not aspirational.

## Non-goals
- Explaining implementation details.

## Contracts
- [index.md](index.md): guarantees onboarding steps are reproducible.
- [../10-architecture/index.md](../10-architecture/index.md): guarantees contract and boundary definitions match code.
- [../20-science/index.md](../20-science/index.md): guarantees stage/tool catalogs match planner/stage contracts.
- [../30-operations/index.md](../30-operations/index.md): guarantees artifact layouts match runtime output.
- [../40-policies/index.md](../40-policies/index.md): guarantees rules enforced by CI.
- [../50-reference/index.md](../50-reference/index.md): guarantees versioning and identifiers are up to date.

Enforcement:
- [../40-policies/POLICY_INDEX.md](../40-policies/POLICY_INDEX.md) tracks policy surfaces that verify docs placement and links.
- [../10-architecture/SNAPSHOT_GOLDEN_CONTRACT.md](../10-architecture/SNAPSHOT_GOLDEN_CONTRACT.md) defines the snapshot outputs that verify contract surfaces.

## Examples
A contract doc must link to the test that enforces it.

## Failure modes
If a promise cannot be enforced, the doc must be removed or rewritten.
