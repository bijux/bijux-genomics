# Boundary Diagram

## What
Canonical boundary diagram for the workspace.

## Why
Single source of truth for architectural boundaries.

## Non-goals
- Describing crate internals (see crate docs).

## Contracts
Enforced by policy tests:
- [BOUNDARY_MAP.md](BOUNDARY_MAP.md) defines the narrative boundary ownership behind this diagram.
- [../../crates/bijux-dna-policies/tests/boundaries/deps/core/dependency_boundaries.rs](../../crates/bijux-dna-policies/tests/boundaries/deps/core/dependency_boundaries.rs)
- [../../crates/bijux-dna-policies/tests/boundaries/deps/graph/effect_boundary_map.rs](../../crates/bijux-dna-policies/tests/boundaries/deps/graph/effect_boundary_map.rs)

## Examples
```
core → runtime → runner
   ↘ engine ↗
        ↘ api
stages ↔ planners ↔ pipelines
analyze ↔ benchmark
```

## Failure modes
Violations fail CI boundary policies.
