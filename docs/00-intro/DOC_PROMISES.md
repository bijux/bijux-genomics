# DOC_PROMISES

## What
Each section declares what is guaranteed to be accurate and how it is enforced.

## Why
Docs must be contract-backed, not aspirational.

## Non-goals
- Explaining implementation details.

## Contracts
- `00-intro`: guarantees onboarding steps are reproducible.
- `10-architecture`: guarantees contract and boundary definitions match code.
- `20-science`: guarantees stage/tool catalogs match planner/stage contracts.
- `30-operations`: guarantees artifact layouts match runtime output.
- `40-policies`: guarantees rules enforced by CI.
- `50-reference`: guarantees versioning and identifiers are up to date.

Enforcement:
- Policy tests verify docs placement and links.
- Snapshot tests verify contract outputs.

## Examples
A contract doc must link to the test that enforces it.

## Failure modes
If a promise cannot be enforced, the doc must be removed or rewritten.
