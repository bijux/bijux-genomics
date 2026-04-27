# Scope

## What
This documentation covers architecture, science, policies, and developer workflows for Bijux Genomics.

## Why
Clear scope prevents doc sprawl and keeps contracts stable.

## Non-goals
- User‑level tutorial for bioinformatics basics.
- Tool‑specific installation guides.

## Contracts
- Docs placement contract (root vs crate docs).

## Examples
- [../10-architecture/index.md](../10-architecture/index.md) describes product architecture and boundaries.
- [../20-science/index.md](../20-science/index.md) describes scientific intent and evidence closure.
- [../30-operations/index.md](../30-operations/index.md) describes runtime and artifact operations.
- [../50-reference/index.md](../50-reference/index.md) describes identifiers, schemas, and compatibility references.
- Crate docs describe crate responsibilities and purity constraints.

## Failure modes
- Mixing product docs inside crates.
- Duplicating contracts in multiple locations.
