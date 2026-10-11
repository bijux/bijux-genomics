# Public API

`bijux-dna-policies` exposes a small policy helper surface. Implementation modules stay private unless they are intentionally listed here and exported from `src/lib.rs`.

## Public Modules
- `policy_diagnostics`
- `public_api`
- `qualification`
- `release_evidence`
- `security_evidence`
- `tool_eligibility`

## Root Exports
- `check`
- `GuardrailConfig`
- `qualification contracts`
- `release evidence contracts`
- `security evidence contracts`
- `tool eligibility decision contracts`
- `CI dependency graph and change-impact planning contracts`
- `CI build authority contracts`
- `policy_assert!`
- `policy_assert_eq!`
- `policy_assert_ne!`
- `policy_panic!`

## Stability Rules
- Root exports are the preferred integration surface.
- `public_api` is a named module for callers that want an explicit namespace.
- `policy_diagnostics` is public because policy failures share a diagnostic contract.
- `checks`, `guardrails`, `source_scan`, and `assertions` are implementation modules.

## Source Authorities
- `src/lib.rs` controls module visibility and root re-exports.
- `src/public_api/stable_surface.rs` curates the callable guardrail surface.
- `src/policy_diagnostics/stable_surface.rs` curates diagnostics exports.
- `security_evidence::derive_security_evidence`: converts complete effect-free scanner,
  structural, and runtime observations into a validated policy publication.
- `release_evidence::derive_release_evidence`: derives immutable release decisions from exact,
  non-self-referential qualification evidence sets.
- `tool_eligibility::select_eligible_tools`: applies an explicit role policy to ordered tool
  candidates without loading registries or reading process state.

`analyze_workspace_dependencies` admits a closed crate adjacency map and returns deterministic cycle components and dependency-first order. The iterative policy has no filesystem or process effects. Cyclic graphs have no complete dependency order; undeclared dependency nodes reject.
