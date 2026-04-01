# Architecture

## Tree
- `src/public_api/` exposes the curated surface for guardrail callers through `stable_surface.rs`.
- `src/guardrails/` owns baseline configuration, crate presets, source inventory setup, and runner wiring.
- `src/checks/` partitions rule families by directory layout, module files, public surface, failure policy, and stage ID literals.
- `src/source_scan/` performs deterministic Rust source discovery through a dedicated stable surface.
- `src/policy_diagnostics/` owns the WHAT/WHY/HOW/MORE diagnostic contract, renderer, and stable surface.
- `src/assertions/` partitions the exported policy macros by condition checks, comparison checks, and panic helpers.

## Data flow
1. `public_api` exposes `check` and `GuardrailConfig`.
2. `guardrails::source_inventory` collects crate sources through `source_scan`.
3. `checks` evaluates structural and content rules.
4. `policy_diagnostics` and `assertions` render uniform failures.
