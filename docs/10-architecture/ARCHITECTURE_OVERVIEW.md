# ARCHITECTURE_OVERVIEW

## What
High-level overview of Bijux architecture.

## Why
Provides a top-down map of contracts and boundaries.

## Non-goals
- Detailed crate internals.

## Contracts
- Boundaries enforced by `crates/bijux-dna-policies/tests/deps/dependency_boundaries.rs`.
- Contracts enforced by `crates/bijux-dna-policies/tests/data/contract_handshake.rs`.

## Examples
See `BOUNDARY_DIAGRAM.md` and `CONTRACT_SPINE.md` for canonical references.

## Failure modes
Violations trigger policy or contract handshake failures.
