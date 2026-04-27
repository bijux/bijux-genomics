# Design Authority

## What
Defines the authority for architectural decisions.

## Why
Avoids conflicting design directives.

## Non-goals
- Governance for unrelated projects.

## Contracts
- Architecture changes must update
  [BOUNDARY_MAP.md](../10-architecture/BOUNDARY_MAP.md),
  [CONTRACT_AUTHORITY_LADDER.md](../10-architecture/CONTRACT_AUTHORITY_LADDER.md), and the
  relevant entries in [POLICY_INDEX.md](../40-policies/POLICY_INDEX.md).

## Examples
- Boundary changes require updating BOUNDARY_MAP.

## Failure modes
- Unapproved drift causes policy failures.
