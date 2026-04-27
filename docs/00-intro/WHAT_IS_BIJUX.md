# WHAT_IS_BIJUX

## What
Bijux is a reproducible bioinformatics pipeline system that turns sequencing inputs into audited outputs with stable contracts.

## Why
Modern pipelines fail when results cannot be reproduced or explained. Bijux makes runs deterministic, contracts explicit, and artifacts traceable.

## Non-goals
- It is not a general workflow engine for arbitrary tasks.
- It does not execute tools without declared contracts.
- It does not hide or auto-mutate outputs.

## Contracts
Three invariants define Bijux:
- **SSOT**: IDs and contract types have a single owner.
- **Determinism**: same inputs ⇒ same outputs (timestamps excluded).
- **Effect boundaries**: only allowlisted crates may spawn processes or access networks.

## Examples
See [QUICKSTART.md](QUICKSTART.md) for a first successful run.
Scope and architecture entrypoints:
- [SCOPE.md](SCOPE.md)
- [../10-architecture/ARCHITECTURE_OVERVIEW.md](../10-architecture/ARCHITECTURE_OVERVIEW.md)
Licensing policy reference: [../50-reference/LICENSING.md](../50-reference/LICENSING.md).

## Failure modes
If any invariant is violated, policies fail in CI and execution halts with a contract error.
