# Architecture

## Tree
- `src/public_api/` exposes the curated surface for guardrail callers.
- `src/guardrails/` owns configuration, crate presets, and runner wiring.
- `src/checks/` partitions rule families by directory layout, module files, public surface, failure policy, and stage ID literals.
- `src/source_scan/` performs deterministic Rust source discovery.
- `src/policy_diagnostics/` owns the WHAT/WHY/HOW/MORE diagnostic contract and renderer.
- `src/assertions.rs` exports the policy assertion macros used by the test suite.

## Data flow
1. `public_api` exposes `check` and `GuardrailConfig`.
2. `guardrails::runner` collects Rust sources through `source_scan`.
3. `checks` evaluates structural and content rules.
4. `policy_diagnostics` and `assertions` render uniform failures.
