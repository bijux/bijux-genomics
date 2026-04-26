# Tests

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
- `tests/schemas/` — reserved for schema-specific policy artifacts.

## Naming
- Policy tests use `policy__<suite>__<file>__<rule>` for stable search and failure triage.
- Source-tree guardrails for this crate live in `tests/boundaries/architecture_tree.rs`.

## Policy Registry
- `tests/boundaries/deps/` enforces dependency budgets, workspace DAG rules, and layer boundaries.
- `tests/boundaries/surface/docs/` enforces docs placement, required docs, docs links, and docs spine rules.
- `tests/boundaries/surface/policy/` enforces policy documentation, style, README, and architecture pointer rules.
- `tests/boundaries/surface/purity/` enforces purity boundaries for core, domain, planner, and stage specs.
- `tests/boundaries/surface/structure_guards/` enforces source layout and naming guardrails.
- `tests/boundaries/surface/structure_layout/` enforces workspace layout, ownership, path, and API boundaries.
- `tests/contracts/tooling/` enforces CI, command, docs, governance, registry, runtime, and stage contracts.

## Verification
Run the package test command from `COMMANDS.md` before handoff. For narrow changes, run the matching `--test` entrypoint plus the full package test before final handoff.
