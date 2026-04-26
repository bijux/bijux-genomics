# Tests

Pipeline profiles are consumed across the stack, so registry drift, defaults drift, dependency drift, and layout drift need explicit locks.

## Entry Points
- `tests/boundaries.rs` — source-tree, docs, dependency, command, effects, and policy guardrails.
- `tests/contracts.rs` — defaults, profile, registry, and snapshot contract coverage.
- `tests/guardrails.rs` — crate-local guardrail smoke coverage.
- `tests/invariant_fast.rs` — fast FASTQ invariant checks.

## Intent Directories
- `tests/boundaries/` — architecture and boundary coverage.
- `tests/contracts/` — defaults, profile, and registry contracts.
- `tests/snapshots/` — checked-in snapshot contracts.

## Source-Tree Contract
`tests/boundaries/architecture_tree.rs` locks the documented source layout, including the split `fastq/{defaults,profiles,invariants}` trees and the `registry/{catalog,families,profile_lookup}` partitions.

## Command Contract
`tests/boundaries/command_inventory.rs` locks that this crate owns no runtime commands and has no `src/bin/` entrypoints.

## Effects Contract
`tests/boundaries/effects_boundary.rs` scans source for forbidden process and network primitives.

## Standard Command
Run:

```bash
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-pipelines --no-default-features
```
