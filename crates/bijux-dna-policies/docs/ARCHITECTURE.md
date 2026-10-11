# Architecture

`bijux-dna-policies` owns pure, deterministic policy decisions and reusable repository guardrails.
It does not own execution effects or domain workflows. Production consumers may use its typed
qualification, publication-freshness, and security-evidence decisions without importing repository
audit orchestration.

## Root Layout

- `Cargo.toml` declares the policy engine dependencies.
- `README.md` is the only root documentation file.
- `docs/` contains the 10 authoritative crate docs.
- `src/` contains reusable policy runner and diagnostic support.
- `tests/` contains executable workspace policies, fixtures, snapshots, and
  shared policy test helpers.

## Source Tree
- `src/public_api/` exposes the curated guardrail caller surface.
- `src/qualification/` owns evidence dimensions, claims, publication freshness, and portable
  qualification reports.
- `src/security_evidence/` owns typed scanner evidence and deterministic security decisions.
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
1. Qualification callers provide admitted evidence decisions; policy models validate complete
   dimensions and derive claims and report summaries.
2. Security callers provide typed scanner publications; policy validation derives admissibility.
3. Repository-audit callers use `check` and `GuardrailConfig`; guardrails collect Rust sources and
   evaluate deterministic source and layout rules.
4. Assertion macros and `policy_diagnostics` render failures with consistent repair guidance.

## Naming
Policy test names use `policy__<suite>__<file>__<rule>` when they enforce workspace governance. Crate-local lock tests may use concise names when they only protect this crate's own tree or manifest.

## Dependency Direction

Production crates may depend on pure qualification, freshness, and security decisions. Repository
guardrails may inspect crate manifests, source trees, docs, reports, and governed fixtures. Policy
code must not own workflow execution, stage planning, runner backends, or CLI routing.

## Change Rules

- Keep reusable rule families in `src/checks/`; keep workspace-specific policy
  assertions in `tests/`.
- Keep diagnostic wording in `src/policy_diagnostics/`.
- Keep deterministic source discovery in `src/source_scan/`.
- Update `tests/boundaries/architecture_tree.rs` and this document together when
  the policy source or test tree changes intentionally.
