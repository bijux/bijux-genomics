# Dependency Rules

## What
Canonical dependency rules for workspace layering.

## Why
Prevent accidental coupling and preserve purity boundaries.

## Non-goals
- Detailed crate design.

## Contracts
Enforced by policy tests:
- `crates/bijux-policies/tests/deps/dependency_graph.rs`
- `crates/bijux-policies/tests/deps/dependency_boundaries.rs`

## Examples
- `bijux-core` must not depend on runners or planners.
- `bijux-runner` must not depend on `bijux-engine`.
- `bijux-api` depends on planners + runtime, not on runner internals.

## Failure modes
Any forbidden edge fails CI dependency policies.
