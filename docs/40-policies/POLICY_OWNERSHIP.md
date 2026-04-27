# POLICY_OWNERSHIP

## What
Defines where policies live and how other docs should reference them.

## Why
Single ownership avoids drift and conflicting guidance.

## Non-goals
- Describing policy implementation details.

## Contracts
Policies live only in [crates/bijux-dna-policies/README.md](../../crates/bijux-dna-policies/README.md).
Other docs must link through [POLICY_INDEX.md](POLICY_INDEX.md) and the authority ledger in
[CONTRACT_INDEX.md](../10-architecture/CONTRACT_INDEX.md), not redefine policy text locally.

## Examples
- Use `policy__surface__docs_spine__...` in docs instead of rewriting rules.

## Failure modes
- Duplicate policy descriptions diverge and cause inconsistent enforcement.
