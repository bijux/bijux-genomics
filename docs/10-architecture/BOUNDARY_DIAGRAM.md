# Boundary Diagram

## What
Canonical boundary diagram for the workspace.

## Why
Single source of truth for architectural boundaries.

## Non-goals
- Describing crate internals (see crate docs).

## Contracts
Enforced by policy tests:
- `crates/bijux-policies/tests/deps/dependency_boundaries.rs`
- `crates/bijux-policies/tests/deps/effect_boundary_map.rs`

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
