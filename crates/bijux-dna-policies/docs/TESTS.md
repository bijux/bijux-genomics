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
- Schema-style policy snapshots stay under `tests/snapshots/` until a materialized schemas suite exists.

## Naming
- Name tests for the behavior they protect; native test IDs include the package, target, and module owner.
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

Shared guardrail parser regressions are owned and executed by the policy crate. This boundary suite retains its crate-specific architecture checks; it does not compile another crate's test source. Constructor-only guardrail wrappers provide no source-check coverage and are retired.

Dependency boundary readers share `tests/support/cargo_dependency_authority.rs`. Each actual
workspace guard captures one explicit-manifest Cargo metadata observation and reuses its
package-indexed dependencies for the complete guard. Aliases retain package identity; normal,
build, dev and target-specific declarations remain in the observation. Physical manifest
lookups refuse missing or foreign packages. Fixture mutation controls intentionally capture
fresh observations; no process-global cache or per-package Cargo invocation is used by the
workspace map guards.

Mandatory guardrails check source acquisition, namespace structure and configured panic or
stage-identifier rules. Equivalent public aliases retain the same outcome whether their
reexports are grouped or separate statements. The public-item count field remains serialized
for configuration compatibility and does not impose a mandatory lexical quota. The separate
public-use count rule remains an explicit opt-in policy with its scoped-reexport negative.
