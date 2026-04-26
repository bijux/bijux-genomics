# Architecture

`bijux-dna-policies` is organized around reusable policy support in `src/` and executable policy rules in `tests/`.

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
- `tests/support/` contains shared filesystem helpers only.

## Data Flow
1. Callers use `check` and `GuardrailConfig` through the public surface.
2. Guardrails collect Rust sources through `source_scan`.
3. `checks` evaluates deterministic source and layout rules.
4. Assertion macros and `policy_diagnostics` render failures with consistent repair guidance.

## Naming
Policy test names use `policy__<suite>__<file>__<rule>` when they enforce workspace governance. Crate-local lock tests may use concise names when they only protect this crate's own tree or manifest.
