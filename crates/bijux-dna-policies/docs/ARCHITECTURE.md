# Architecture

`bijux-dna-policies` owns pure, deterministic policy decisions and reusable repository guardrails.
It does not own execution effects or domain workflows. Production consumers may use its typed
qualification, publication-freshness, security-evidence, and release-evidence decisions without
importing repository audit orchestration.

## Root Layout

- `Cargo.toml` declares the policy engine dependencies.
- `README.md` is the only root documentation file.
- `docs/` contains the 10 authoritative crate docs.
- `src/` contains reusable policy runner and diagnostic support.
- `tests/` contains executable workspace policies, fixtures, snapshots, and
  shared policy test helpers.

## Source Tree
- `src/public_api/` exposes the curated guardrail caller surface.
- `src/qualification/` owns evidence dimensions, claims, publication freshness, portable
  qualification reports, and deterministic reviewer-facing rendering.
- `src/release_evidence/` owns deterministic release derivation, evidence-set identity, strict
  admission, and policy decisions without performing publication I/O.
- `src/security_evidence/` owns typed scanner evidence, pure observation derivation, and
  deterministic security decisions.
- `src/tool_eligibility/` owns pure role-based tool admission decisions over caller-supplied
  candidates; registry loading and environment adaptation remain with composition surfaces.
- `src/guardrails/` owns baseline configuration, crate presets, source inventory setup, and runner wiring.
- `src/checks/` partitions reusable rule families by build authority, dependency graph,
  non-waiting execution, directory layout, module files, public surface, failure policy, and stage
  ID literals.
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
- `tests/support/` owns private filesystem, Cargo-observation, and Rust source classification helpers.
- `tests/support/rust_source/scratch_paths.rs` classifies cwd-derived filesystem path admission;
  the contracts policy owns its counterexamples and workspace-wide enforcement.

## Data Flow
1. Qualification callers provide admitted observations; policy rules record versioned measured
   values, thresholds, outcomes, reasons, and evidence identities before models derive claims and
   report summaries.
2. Security callers provide catalog-complete typed observations. Policy validation keeps product
   support, artifact delivery, redistribution permission, and artifact availability independent;
   a supported external tool remains in the publication when access is blocked, while an admitted
   external artifact can carry the same artifact-bound controls as a redistributable image.
3. Release callers bind every tool to the exact non-release qualification evidence set. Admission
   rejects authority, coverage, support-dimension, evidence, or policy-verdict drift.
4. CI callers provide the build authority, dependency graph, and changed paths; policy code
   validates toolchain, lock, cache-metric, target-isolation, and graph integrity and computes a
   fail-closed reverse-dependent gate plan without executing it.
5. Tool-selection callers provide ordered candidate identities, eligibility tiers, and explicit
   admission options; policy code returns the stable eligible order or a fail-closed verdict.
6. Repository-audit callers use `check` and `GuardrailConfig`; guardrails collect Rust sources and
   evaluate deterministic source and layout rules.
7. Assertion macros and `policy_diagnostics` render failures with consistent repair guidance.

## Naming
Test names describe the behavior they protect. Native test IDs retain the package, target, and module owner without repeating the source path in each function name.

## Dependency Direction

Production crates may depend on pure qualification, freshness, security, and release decisions.
Repository guardrails may inspect crate manifests, source trees, docs, reports, and governed
fixtures. Policy code must not own workflow execution, stage planning, runner backends, or CLI
routing.

## Change Rules

- Keep reusable rule families in `src/checks/`; keep workspace-specific policy
  assertions in `tests/`.
- Keep diagnostic wording in `src/policy_diagnostics/`.
- Keep deterministic source discovery in `src/source_scan/`.
- Update `tests/boundaries/architecture_tree.rs` and this document together when
  the policy source or test tree changes intentionally.
