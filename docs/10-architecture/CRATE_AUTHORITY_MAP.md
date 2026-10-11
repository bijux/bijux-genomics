# Crate Authority Map

## Purpose

Provide a compatibility navigation surface for crate ownership checks without maintaining a second
responsibility table.

## Authority

The repository-wide ownership source of truth is
[TARGET_ARCHITECTURE.md](../../TARGET_ARCHITECTURE.md). It defines the exclusive container and
benchmark product owners, supporting crate boundaries, generic lifecycle, and thin CLI/operator
rules.

Allowed dependency edges remain authoritative in
[BOUNDARY_MAP.md](BOUNDARY_MAP.md). Repository policy diagnostics are documented in
[bijux-dna-policies enforcement](../../crates/bijux-dna-policies/docs/ENFORCEMENT.md).

## Enforcement

- Dependency edges:
  [dependency_boundaries.rs](../../crates/bijux-dna-policies/tests/boundaries/deps/core/dependency_boundaries.rs)
- Dependency graph:
  [dependency_graph.rs](../../crates/bijux-dna-policies/tests/boundaries/deps/graph/dependency_graph.rs)
- Effect boundaries:
  [effect_boundary_map.rs](../../crates/bijux-dna-policies/tests/boundaries/deps/graph/effect_boundary_map.rs)
- Command spawn confinement:
  [command_spawn_policy.rs](../../crates/bijux-dna-policies/tests/contracts/tooling/governance_core/command_spawn_policy.rs)
- Domain, stage, and crate responsibility checks:
  [purity_effects_responsibility_policy.rs](../../crates/bijux-dna-policies/tests/contracts/tooling/governance/purity_effects_responsibility_policy.rs)

## Failure Mode

Adding responsibility prose here would create parallel ownership authority. Update the target
architecture and its executable checks together instead.
