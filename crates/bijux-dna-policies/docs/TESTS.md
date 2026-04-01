# Tests

## What
Maps the stable policy test entrypoints and their intent directories.

## Why
The suite is large; stable aggregator files and directory guides are the durable map.

## Entry points
- `tests/boundaries.rs` — aggregates boundary, surface, documentation, and workspace layout policies.
- `tests/contracts.rs` — aggregates contract, tooling, fixture, and governance policies.
- `tests/determinism.rs` — aggregates deterministic fixture and workspace-order checks.
- `tests/guardrails.rs` — smoke-checks crate-specific guardrail wiring.

## Intent directories
- `tests/boundaries/` — dependency boundaries, source layout, documentation spine, and surface safety rules.
- `tests/contracts/` — policy contracts, snapshots, tooling governance, and workspace-level invariants.
- `tests/determinism/` — stable-order and fixture reproducibility rules.
- `tests/support/` — shared filesystem helpers used across policy suites.
- `tests/fixtures/` — sample inputs referenced by policy tests.
- `tests/snapshots/` — locked snapshots for guardrail defaults and documentation contracts.
- `tests/schemas/` — reserved for schema-specific suite docs and serialized artifacts.

## Naming
- Policy tests use `policy__<suite>__<file>__<rule>` for stable search and failure triage.
- Source-tree guardrails for this crate live in `tests/boundaries/architecture_tree.rs`.
