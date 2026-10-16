# Crate Authority Map

## Purpose

Provide a compatibility navigation surface for crate ownership checks without maintaining a second
responsibility table.

## Scope

Use this page to find the authority for a crate responsibility, dependency edge,
or ownership-check failure. It connects the product ownership contract, dependency
map, and executable checks; the linked documents retain their separate roles.

## Non-goals

- Maintaining another crate responsibility table or an alternative dependency graph.
- Granting a dependency exception because a crate appears in this navigation page.
- Replacing crate-local API and effect contracts with repository-wide prose.

## Contracts

- Resolve product and lifecycle ownership in
  [the target architecture](TARGET_ARCHITECTURE.md), rather than deriving an owner
  from a command name, directory location, or this page's link list.
- Resolve permitted dependency edges in [the boundary map](BOUNDARY_MAP.md).
  A responsibility assignment does not by itself authorize a Cargo dependency.
- Follow the matching enforcement link below when a dependency, effect, command
  spawn, or responsibility check fails. Change the owning contract and executable
  check together when the intended boundary changes.

## Authority

The repository-wide ownership source of truth is
[TARGET_ARCHITECTURE.md](TARGET_ARCHITECTURE.md). It defines the exclusive container and
benchmark product owners, supporting crate boundaries, generic lifecycle, and thin CLI/operator
rules.

Allowed dependency edges remain authoritative in
[BOUNDARY_MAP.md](BOUNDARY_MAP.md). Repository policy diagnostics are documented in
[bijux-dna-policies enforcement](https://github.com/bijux/bijux-genomics/blob/main/crates/bijux-dna-policies/docs/ENFORCEMENT.md).

## Enforcement

- Dependency edges:
  [dependency_boundaries.rs](https://github.com/bijux/bijux-genomics/blob/main/crates/bijux-dna-policies/tests/boundaries/deps/core/dependency_boundaries.rs)
- Dependency graph:
  [dependency_graph.rs](https://github.com/bijux/bijux-genomics/blob/main/crates/bijux-dna-policies/tests/boundaries/deps/graph/dependency_graph.rs)
- Effect boundaries:
  [effect_boundary_map.rs](https://github.com/bijux/bijux-genomics/blob/main/crates/bijux-dna-policies/tests/boundaries/deps/graph/effect_boundary_map.rs)
- Command spawn confinement:
  [command_spawn_policy.rs](https://github.com/bijux/bijux-genomics/blob/main/crates/bijux-dna-policies/tests/contracts/tooling/governance_core/command_spawn_policy.rs)
- Domain, stage, and crate responsibility checks:
  [purity_effects_responsibility_policy.rs](https://github.com/bijux/bijux-genomics/blob/main/crates/bijux-dna-policies/tests/contracts/tooling/governance/purity_effects_responsibility_policy.rs)

## Failure Mode

Adding responsibility prose here would create parallel ownership authority. Update the target
architecture and its executable checks together instead.
