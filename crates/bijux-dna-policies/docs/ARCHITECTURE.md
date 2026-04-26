# Architecture

`bijux-dna-policies` is organized around reusable policy support in `src/` and
executable policy rules in `tests/`. It is a governance crate: production crates
may use its public policy runner only when they are intentionally performing a
workspace audit, and most crates should depend on it only from tests.

## Root Layout

- `Cargo.toml` declares the policy engine dependencies.
- `README.md` is the only root documentation file.
- `docs/` contains the 10 authoritative crate docs.
- `src/` contains reusable policy runner and diagnostic support.
- `tests/` contains executable workspace policies, fixtures, snapshots, and
  shared policy test helpers.

## Source Tree
- `src/public_api/` exposes the curated guardrail caller surface.
- `src/guardrails/` owns baseline configuration, crate presets, source inventory setup, and runner wiring.
- `src/checks/` partitions reusable rule families by directory layout, module files, public surface, failure policy, and stage ID literals.
- `src/source_scan/` performs deterministic Rust source discovery.
- `src/policy_diagnostics/` owns the WHAT/WHY/HOW/MORE diagnostic contract and renderer.
- `src/assertions/` owns exported assertion macros.

## Test Tree
- `tests/boundaries.rs` aggregates dependency, surface, layout, documentation, and workspace boundary policies.
- `tests/contracts.rs` aggregates contract, tooling, fixture, and governance policies.
- `tests/determinism.rs` aggregates stable-order and fixture reproducibility policies.
- `tests/guardrails.rs` smoke-checks crate-specific guardrail wiring.
- `tests/fixtures/` contains governed fixtures for policy tests.
- `tests/snapshots/` contains governed diagnostic snapshots.
- `tests/support/` contains shared filesystem helpers only.

## Data Flow
1. Callers use `check` and `GuardrailConfig` through the public surface.
2. Guardrails collect Rust sources through `source_scan`.
3. `checks` evaluates deterministic source and layout rules.
4. Assertion macros and `policy_diagnostics` render failures with consistent repair guidance.

## Naming
Policy test names use `policy__<suite>__<file>__<rule>` when they enforce workspace governance. Crate-local lock tests may use concise names when they only protect this crate's own tree or manifest.

## Dependency Direction

Policy code may inspect crate manifests, source trees, docs, generated reports,
and governed fixture data. It must not become a production runtime dependency
for normal workflow execution, stage planning, runner backends, or CLI behavior
unless the caller is explicitly running a policy audit.

## Change Rules

- Keep reusable rule families in `src/checks/`; keep workspace-specific policy
  assertions in `tests/`.
- Keep diagnostic wording in `src/policy_diagnostics/`.
- Keep deterministic source discovery in `src/source_scan/`.
- Update `tests/boundaries/architecture_tree.rs` and this document together when
  the policy source or test tree changes intentionally.
