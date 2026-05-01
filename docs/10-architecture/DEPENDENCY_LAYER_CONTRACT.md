# Dependency Layer Contract

Owner: Architecture  
Scope: workspace dependency direction and exception handling  
Last reviewed: 2026-04-30  
Contract version: v1

## Purpose

State the allowed dependency direction once, then make drift visible in both policy failures and a committed cargo-metadata snapshot.

## Layer rule

The workspace is intended to flow in this direction:

`foundation -> domain -> stage-contract -> planners -> stages -> runtime -> runner -> engine -> api -> cli`

Consumers such as `analyze`, `bench`, `science`, `qa`, and `dev` may read artifacts,
plans, manifests, and evidence, but they must not become hidden execution authorities.

## Exception process

When a crate needs a dependency that appears to violate the layer rule:

1. document the reason in the pull request;
2. update [CRATE_RESPONSIBILITY_MATRIX.md](CRATE_RESPONSIBILITY_MATRIX.md) if ownership moved;
3. update the committed cargo metadata snapshot only after the new edge is justified;
4. if the edge is temporary, record the removal condition in the PR handoff.

No undocumented dependency exception is allowed.

## Enforcement

- Hard dependency policies still live in
  `crates/bijux-dna-policies/tests/boundaries/deps/core/dependency_boundaries.rs`
  and
  `crates/bijux-dna-policies/tests/boundaries/deps/graph/dependency_graph.rs`.
- The committed snapshot lives in
  `crates/bijux-dna-policies/tests/fixtures/cargo_metadata_snapshot/workspace-deps.txt`.
- The snapshot validator is
  `crates/bijux-dna-policies/tests/contracts/tooling/governance_core/cargo_metadata_snapshot_policy.rs`.

## Scope
This document defines the operational or architecture surface for this workflow surface.

## Non-goals
- Replacing crate-level implementation details or test contracts.

## Contracts
- Changes to this surface must stay aligned with governed checks and generated references.
