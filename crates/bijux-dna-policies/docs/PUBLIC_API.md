# Public API

`bijux-dna-policies` exposes a small policy helper surface. Implementation modules stay private unless they are intentionally listed here and exported from `src/lib.rs`.

## Public Modules
- `policy_diagnostics`
- `public_api`

## Root Exports
- `check`
- `GuardrailConfig`
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
